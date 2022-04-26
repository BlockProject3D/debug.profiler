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

use std::io::{Read, Write};
use std::net::TcpStream;
use bincode::ErrorKind;
use crossbeam_channel::Receiver;
use druid::{ExtEventSink, Target};
use crate::command::{CONNECTION_ERROR, CONNECTION_SUCCESS, NETWORK_COMMAND, NETWORK_ERROR};
use crate::constants::NET_READ_DURATION;
use crate::thread::base::{BaseWorker, Run};
use crate::thread::network_types::{Hello, HELLO_PACKET, MatchResult};
use super::network_types::Command as NetCommand;

fn handle_hello(socket: &mut TcpStream) -> Result<(), String> {
    let mut block = [0; 40];
    socket.read_exact(&mut block).map_err(|e| e.to_string())?;
    let msg = Hello::from_bytes(block);
    match HELLO_PACKET.matches(&msg) {
        MatchResult::SignatureMismatch => return Err(format!("protocol signature mismatch ({:?})",
                                                             msg.signature)),
        MatchResult::VersionMismatch => return Err(format!("protocol version mismatch ({} - {:?})",
                                                           msg.version.major,
                                                           msg.version.pre_release)),
        MatchResult::Ok => ()
    }
    socket.write_all(&HELLO_PACKET.to_bytes()).map_err(|e| e.to_string())?;
    Ok(())
}

struct Worker {
    stream: TcpStream
}

impl Worker {
    pub fn new(stream: TcpStream) -> Worker {
        Worker {
            stream
        }
    }

    pub fn connect(ip: String) -> Result<TcpStream, String> {
        let mut socket = TcpStream::connect(&ip).map_err(|e| e.to_string())?;
        socket.set_read_timeout(Some(NET_READ_DURATION)).map_err(|e| e.to_string())?;
        handle_hello(&mut socket)?;
        Ok(socket)
    }

    pub fn try_read_command(&mut self) -> Result<Option<NetCommand>, String> {
        match bincode::deserialize_from(&mut self.stream) {
            Ok(v) => Ok(Some(v)),
            Err(e) => {
                if let ErrorKind::Io(e) = &*e {
                    if e.kind() == std::io::ErrorKind::WouldBlock
                        || e.kind() == std::io::ErrorKind::TimedOut {
                        return Ok(None)
                    }
                }
                Err(e.to_string())
            }
        }
    }

    pub fn read_command(&mut self, base: &BaseWorker<Internal>) -> bool {
        match self.try_read_command() {
            Ok(v) => {
                if let Some(v) = v {
                    base.send(Ok(v));
                    return false
                }
            },
            Err(e) => {
                base.send(Err(e));
                return true
            }
        }
        false
    }
}

impl Run<Internal> for Worker {
    fn run(&mut self, base: &BaseWorker<Internal>) {
        loop {
            if self.read_command(base) || base.should_exit() {
                break
            }
        }
    }
}

struct Internal {
    buffer: super::command::Buffer
}

type Params = (String, Option<usize>);

impl super::base::Connection for Internal {
    type Message = Result<NetCommand, String>;
    type Worker = Worker;
    type Parameters = Params;

    fn max_messages(&self) -> usize {
        self.buffer.max_buffer()
    }

    fn new(sink: &ExtEventSink, (ip, max_sub_buffer): Params) -> Option<(Self, Self::Worker)> {
        let socket = match Worker::connect(ip) {
            Ok(v) => v,
            Err(e) => {
                sink.submit_command(CONNECTION_ERROR, e, Target::Auto).unwrap();
                return None;
            }
        };
        sink.submit_command(CONNECTION_SUCCESS, false, Target::Auto).unwrap();
        let this = Internal {
            buffer: super::command::Buffer::new(max_sub_buffer)
        };
        let worker = Worker::new(socket);
        Some((this, worker))
    }

    fn step(&mut self, sink: &ExtEventSink, channel: &Receiver<Self::Message>) -> bool {
        if let Err(e) = self.buffer.try_submit(channel) {
            sink.submit_command(NETWORK_ERROR, e, Target::Auto).unwrap();
            return false;
        }
        if let Some(batch) = self.buffer.fast_forward() {
            sink.submit_command(NETWORK_COMMAND, batch, Target::Auto).unwrap();
        }
        if self.buffer.should_terminate() {
            return false;
        }
        true
    }
}

super::base::hack_rust_garbage_private_rule!(Connection<Params, Internal>);
