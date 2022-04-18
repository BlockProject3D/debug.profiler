// Copyright (c) 2022, BlockProject 3D
//
// All rights reserved.
//
// Redistribution and use in source and binary forms, with or without modification,
// are permitted provided that the following conditions are met:
//
//     * Redistributions of source code must retain the above copyright notice,
//       this list of conditions and the following disclaimer.
//     * Redistributions in binary form must reproduce the above copyright notice,
//       this list of conditions and the following disclaimer in the documentation
//       and/or other materials provided with the distribution.
//     * Neither the name of BlockProject 3D nor the names of its contributors
//       may be used to endorse or promote products derived from this software
//       without specific prior written permission.
//
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
// "AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
// LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
// A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT OWNER OR
// CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL,
// EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO,
// PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR
// PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF
// LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING
// NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS
// SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.

use std::io::ErrorKind;
use std::net::{Ipv4Addr, UdpSocket};
use crossbeam_channel::Receiver;
use druid::{ExtEventSink, Target};
use druid::im::HashSet;
use crate::{DEFAULT_PORT, PROTOCOL_VERSION};
use crate::command::{NETWORK_PEER, NETWORK_PEER_ERR};
use crate::state::Peer;
use crate::thread::base::{BaseWorker, Connection, Run};
use crate::thread::NET_READ_DURATION;

// The maximum number of characters allowed for the application name in the auto-discover list.
const NAME_MAX_CHARS: usize = 126;

// The maximum number of pending received broadcast packets.
const MAX_BUFFER: usize = 64;

const PROTOCOL_SIGNATURE: u8 = b'B';

struct Worker {
    socket: UdpSocket,
    set: HashSet<String>
}

impl Worker {
    pub fn new() -> std::io::Result<Worker> {
        // Create the UDP connection that will be used to receive auto-discover LAN broadcast
        // packets.
        let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, DEFAULT_PORT))?;
        socket.set_read_timeout(Some(NET_READ_DURATION))?;
        Ok(Worker {
            socket,
            set: HashSet::new()
        })
    }

    pub fn try_read_peer(&mut self) -> Result<Option<Peer>, String> {
        let mut buffer: [u8; NAME_MAX_CHARS + 2] = [0; NAME_MAX_CHARS + 2];
        match self.socket.recv_from(&mut buffer) {
            Ok((len, peer_addr)) => {
                if len != NAME_MAX_CHARS + 2 || buffer[0] != PROTOCOL_SIGNATURE
                    || buffer[1] != PROTOCOL_VERSION {
                    // The message is not valid for this application.
                    return Ok(None);
                }
                let str = buffer[2..].split(|v| *v == 0).next().unwrap_or(&[]);
                match std::str::from_utf8(str) {
                    Ok(name) => {
                        if self.set.contains(name) {
                            Ok(None)
                        } else {
                            let peer = Peer {
                                addr: peer_addr.ip(),
                                name: name.into()
                            };
                            self.set.insert(name.into());
                            Ok(Some(peer))
                        }
                    }
                    Err(_) => Ok(None)
                }
            },
            Err(e) => {
                if e.kind() == ErrorKind::WouldBlock || e.kind() == ErrorKind::TimedOut {
                    return Ok(None);
                }
                Err(e.to_string())
            }
        }
    }
}

impl Run<Internal> for Worker {
    fn run(&mut self, base: &BaseWorker<Internal>) {
        loop {
            if base.should_exit() {
                break;
            }
            if let Some(res) = self.try_read_peer().transpose() {
                base.send(res);
            }
        }
    }
}

struct Internal;

impl Connection for Internal {
    type Message = Result<Peer, String>;
    type Worker = Worker;
    type Parameters = ();
    const MAX_MESSAGES: usize = MAX_BUFFER;

    fn new(_: &ExtEventSink, _: Self::Parameters) -> Option<(Self, Self::Worker)> {
        let worker = Worker::new().ok()?;
        Some((Internal, worker))
    }

    fn step(&mut self, sink: &ExtEventSink, channel: &Receiver<Self::Message>) -> bool {
        while let Ok(v) = channel.try_recv() {
            match v {
                Ok(v) => sink.submit_command(NETWORK_PEER, v, Target::Auto).unwrap(),
                Err(e) => {
                    sink.submit_command(NETWORK_PEER_ERR, e, Target::Auto).unwrap();
                    return false;
                }
            }
        }
        return true;
    }
}

super::base::hack_rust_garbage_private_rule!(AutoDiscoveryConnection<(), Internal>);
