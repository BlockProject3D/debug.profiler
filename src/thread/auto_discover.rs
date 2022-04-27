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

use druid::{ExtEventSink, Target};
use std::collections::HashSet;
use std::net::Ipv4Addr;
use crate::command::{NETWORK_PEER, NETWORK_PEER_ERR};
use crate::constants::{AUTODISCOVERY_PROTOCOL_VERSION, DEFAULT_PORT};
use crate::state::Peer;
use crate::thread::base::{BaseConnection, Connection, Run};
use async_trait::async_trait;
use tokio::net::UdpSocket;
use tokio::sync::mpsc::{Receiver, Sender};

// The maximum number of characters allowed for the application name in the auto-discover list.
const NAME_MAX_CHARS: usize = 126;

// The maximum number of pending received broadcast packets.
const MAX_BUFFER: usize = 64;

const PROTOCOL_SIGNATURE: u8 = b'B';

type Message = Result<Peer, String>;

struct Worker {
    set: HashSet<String>,
    channel: Sender<Message>
}

impl Worker {
    pub fn new(channel: Sender<Message>) -> Worker {
        Worker {
            set: HashSet::new(),
            channel
        }
    }

    async fn read_peer(&mut self, socket: &mut UdpSocket) -> Result<Option<Peer>, String> {
        let mut buffer: [u8; NAME_MAX_CHARS + 2] = [0; NAME_MAX_CHARS + 2];
        let (len, peer_addr) = socket.recv_from(&mut buffer).await.map_err(|e| e.to_string())?;
        if len != NAME_MAX_CHARS + 2 || buffer[0] != PROTOCOL_SIGNATURE
            || buffer[1] != AUTODISCOVERY_PROTOCOL_VERSION {
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
    }
}

#[async_trait]
impl Run for Worker {
    async fn run(&mut self) {
        let mut socket = match UdpSocket::bind((Ipv4Addr::UNSPECIFIED, DEFAULT_PORT)).await {
            Ok(v) => v,
            Err(e) => {
                self.channel.send(Err(e.to_string())).await.unwrap();
                return;
            }
        };
        loop {
            match self.read_peer(&mut socket).await {
                Ok(v) => match v {
                    Some(v) => self.channel.send(Ok(v)).await.unwrap(),
                    None => ()
                },
                Err(e) => self.channel.send(Err(e)).await.unwrap()
            }
        }
    }
}

struct Core {
    channel: Receiver<Message>,
    sink: ExtEventSink
}

#[async_trait]
impl Run for Core {
    async fn run(&mut self) {
        while let Some(msg) = self.channel.recv().await {
            match msg {
                Ok(v) => self.sink.submit_command(NETWORK_PEER, v, Target::Auto).unwrap(),
                Err(e) => {
                    self.sink.submit_command(NETWORK_PEER_ERR, e, Target::Auto).unwrap();
                    break;
                }
            };
        }
    }
}

struct Internal;

impl Connection for Internal {
    type Message = Message;
    type Worker = Worker;
    type Core = Core;
    type Parameters = ();

    fn max_messages(&self) -> usize {
        MAX_BUFFER
    }

    fn new_worker(&self, channel: Sender<Self::Message>) -> Self::Worker {
        Worker::new(channel)
    }

    fn new_core(&self, sink: ExtEventSink, channel: Receiver<Self::Message>) -> Self::Core {
        Core { channel, sink }
    }

    fn new(_: Self::Parameters) -> Option<Self> {
        Some(Internal)
    }
}

pub fn new(sink: ExtEventSink, params: ()) -> Option<BaseConnection> {
    BaseConnection::new::<Internal>(sink, params)
}
