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

use std::net::TcpStream;
use std::sync::mpsc::{Receiver, TryRecvError};
use std::time::Duration;
use bincode::ErrorKind;
use druid::{ExtEventSink, Target};
use crate::command::{CONNECTION_ERROR, CONNECTION_SUCCESS, NETWORK_COMMAND, NETWORK_ERROR};
use crate::network_types::Command as NetCommand;

pub enum Command {
    Connect {
        ip: String,
        sink: ExtEventSink
    },
    Terminate
}

struct Worker {
    stream: TcpStream,
    sink: ExtEventSink
}

impl Worker {
    pub fn new(stream: TcpStream, sink: ExtEventSink) -> Worker {
        sink.submit_command(CONNECTION_SUCCESS, false, Target::Auto).unwrap();
        Worker {
            stream,
            sink
        }
    }

    pub fn connect(ip: String) -> Result<TcpStream, String> {
        let socket = TcpStream::connect(&ip).map_err(|e| e.to_string())?;
        socket.set_read_timeout(Some(Duration::from_millis(500))).map_err(|e| e.to_string())?;
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

    pub fn read_command(&mut self) {
        match self.try_read_command() {
            Ok(v) => {
                if let Some(v) = v {
                    self.sink.submit_command(NETWORK_COMMAND, v, Target::Auto).unwrap();
                }
            },
            Err(e) => {
                self.sink.submit_command(NETWORK_ERROR, e, Target::Auto).unwrap();
            }
        }
    }
}

pub struct NetworkThread {
    channel: Receiver<Command>
}

impl NetworkThread {
    pub fn new(channel: Receiver<Command>) -> NetworkThread {
        NetworkThread {
            channel
        }
    }

    pub fn run(&self) {
        let mut worker = None;
        loop {
            let cmd = match self.channel.try_recv() {
                Ok(v) => Some(v),
                Err(e) => {
                    match e {
                        TryRecvError::Empty => None,
                        TryRecvError::Disconnected => break
                    }
                }
            };
            if let Some(cmd) = cmd {
                match cmd {
                    Command::Connect { ip, sink } => {
                        if worker.is_some() {
                            continue;
                        }
                        let socket1 = match Worker::connect(ip) {
                            Ok(v) => v,
                            Err(e) => {
                                sink.submit_command(CONNECTION_ERROR, e, Target::Auto).unwrap();
                                continue;
                            }
                        };
                        worker = Some(Worker::new(socket1, sink));
                    },
                    Command::Terminate => break
                }
            }
            if let Some(worker) = &mut worker {
                worker.read_command();
            }
            std::thread::sleep(Duration::from_millis(50));
        }
    }
}
