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
use std::net::{IpAddr, Ipv4Addr, UdpSocket};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;
use crossbeam_channel::{bounded, Receiver, Sender};
use druid::{ExtEventSink, Target};
use crate::{DEFAULT_PORT, PROTOCOL_VERSION};
use crate::command::{NETWORK_PEER, NETWORK_PEER_ERR};
use crate::thread::NET_READ_DURATION;

// The maximum number of characters allowed for the application name in the auto-discover list.
const NAME_MAX_CHARS: usize = 128;

// The maximum number of pending received broadcast packets.
const MAX_BUFFER: usize = 128;

const PROTOCOL_SIGNATURE: u8 = b'B';

pub struct Peer {
    pub name: String,
    pub addr: IpAddr
}

struct Worker {
    channel: Sender<Result<Peer, String>>,
    socket: UdpSocket,
    exit_flag: Arc<AtomicBool>
}

impl Worker {
    pub fn new(channel: Sender<Result<Peer, String>>, exit_flag: Arc<AtomicBool>) -> std::io::Result<Worker> {
        // Create the UDP connection that will be used to receive auto-discover LAN broadcast
        // packets.
        let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, DEFAULT_PORT))?;
        socket.set_read_timeout(Some(NET_READ_DURATION))?;
        Ok(Worker {
            channel,
            socket,
            exit_flag
        })
    }

    pub fn try_read_peer(&self) -> Result<Option<Peer>, String> {
        let mut buffer: [u8; NAME_MAX_CHARS + 2] = [0; NAME_MAX_CHARS + 2];
        match self.socket.recv_from(&mut buffer) {
            Ok((len, peer_addr)) => {
                if len <= 2 || buffer[0] != PROTOCOL_SIGNATURE || buffer[1] != PROTOCOL_VERSION {
                    // The message is not valid for this application.
                    return Ok(None);
                }
                match std::str::from_utf8(&buffer[2..len]) {
                    Ok(name) => {
                        let peer = Peer {
                            addr: peer_addr.ip(),
                            name: name.into()
                        };
                        Ok(Some(peer))
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

    pub fn run(&self) {
        loop {
            if self.exit_flag.load(Ordering::Relaxed) {
                break;
            }
            if let Some(res) = self.try_read_peer().transpose() {
                self.channel.send(res).unwrap();
            }
        }
    }
}

pub struct AutoDiscoveryConnection {
    channel: Receiver<Result<Peer, String>>,
    thread_handle: JoinHandle<()>,
    exit_flag: Arc<AtomicBool>,
    sink: ExtEventSink
}

impl AutoDiscoveryConnection {
    pub fn new(sink: ExtEventSink) -> Option<AutoDiscoveryConnection> {
        let (sender, receiver) = bounded(MAX_BUFFER);
        let exit_flag = Arc::new(AtomicBool::new(false));
        let thread = Worker::new(sender, exit_flag.clone()).ok()?;
        let thread_handle = std::thread::spawn(move || thread.run());
        Some(AutoDiscoveryConnection {
            channel: receiver,
            exit_flag,
            thread_handle,
            sink
        })
    }

    pub fn step(self) -> Option<Self> {
        while let Ok(v) = self.channel.try_recv() {
            match v {
                Ok(v) => self.sink.submit_command(NETWORK_PEER, v, Target::Auto).unwrap(),
                Err(e) => {
                    self.sink.submit_command(NETWORK_PEER_ERR, e, Target::Auto).unwrap();
                    return None;
                }
            }
        }
        Some(self)
    }

    pub fn end(self) {
        while self.channel.try_recv().is_ok() {} //Force empty the channel before attempting to join
        self.exit_flag.store(true, Ordering::Relaxed);
        self.thread_handle.join().unwrap();
    }
}
