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

use std::time::Duration;
use tokio::sync::mpsc::Receiver;
use druid::{ExtEventSink, Target};
use tokio::sync::mpsc::Sender;
use crate::command::{CONNECTION_ERROR, CONNECTION_SUCCESS, NETWORK_COMMAND, NETWORK_ERROR};
use crate::constants::DEFAULT_SINGLE_COMMAND_BUFFER;
use crate::thread::base::{BaseConnection, Run};
use crate::thread::network_types::{Hello, HELLO_PACKET, MatchResult};
use super::network_types::Command as NetCommand;
use async_trait::async_trait;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;

async fn handle_hello(socket: &mut TcpStream) -> Result<(), String> {
    let mut block = [0; 40];
    socket.read_exact(&mut block).await.map_err(|e| e.to_string())?;
    let msg = Hello::from_bytes(block);
    match HELLO_PACKET.matches(&msg) {
        MatchResult::SignatureMismatch => return Err(format!("protocol signature mismatch ({:?})",
                                                             msg.signature)),
        MatchResult::VersionMismatch => return Err(format!("protocol version mismatch ({} - {:?})",
                                                           msg.version.major,
                                                           msg.version.pre_release)),
        MatchResult::Ok => ()
    }
    socket.write_all(&HELLO_PACKET.to_bytes()).await.map_err(|e| e.to_string())?;
    Ok(())
}

#[derive(Debug)]
enum Message {
    ConnectionSuccess,
    ConnectionError(String),
    NetworkError(String),
    NetworkCommand(NetCommand)
}

impl Message {
    pub fn into_result(self) -> Result<NetCommand, String> {
        match self {
            Message::ConnectionSuccess => unreachable!(),
            Message::ConnectionError(_) => unreachable!(),
            Message::NetworkError(e) => Err(e),
            Message::NetworkCommand(v) => Ok(v)
        }
    }
}

struct Worker {
    channel: Sender<Message>,
    ip: String,
    command_buffer: Vec<u8>
}

impl Worker {
    pub fn new(ip: String, channel: Sender<Message>) -> Worker {
        Worker {
            ip,
            channel,
            command_buffer: vec![0; DEFAULT_SINGLE_COMMAND_BUFFER]
        }
    }

    async fn connect(&self) -> Result<TcpStream, String> {
        let mut stream = TcpStream::connect(&self.ip).await.map_err(|e| e.to_string())?;
        handle_hello(&mut stream).await?;
        Ok(stream)
    }

    async fn handle_frame(&mut self, res: Result<NetCommand, String>) -> bool {
        match res {
            Ok(v) => {
                self.channel.send(Message::NetworkCommand(v)).await.unwrap();
                false
            }
            Err(e) => {
                self.channel.send(Message::NetworkError(e)).await.unwrap();
                true
            }
        }
    }

    async fn read_command(&mut self, buffer: &mut BufReader<TcpStream>) -> Result<NetCommand, String> {
        let size = buffer.read_u32_le().await.map_err(|e| e.to_string())? as usize;
        if size > self.command_buffer.len() {
            self.command_buffer.resize(size, 0);
        }
        buffer.read_exact(&mut self.command_buffer[..size]).await.map_err(|e| e.to_string())?;
        let cmd: NetCommand = bincode::deserialize(&self.command_buffer[..size]).map_err(|e| e.to_string())?;
        Ok(cmd)
    }
}

#[async_trait]
impl Run for Worker {
    async fn run(&mut self) {
        let stream = match self.connect().await {
            Ok(v) => v,
            Err(e) => {
                self.channel.send(Message::ConnectionError(e)).await.unwrap();
                return;
            }
        };
        self.channel.send(Message::ConnectionSuccess).await.unwrap();
        let mut buffer = BufReader::new(stream);
        loop {
            let cmd = self.read_command(&mut buffer).await;
            if self.handle_frame(cmd).await {
                break;
            }
        }
    }
}

struct Core {
    buffer: super::command::Buffer,
    channel: Receiver<Message>,
    sink: ExtEventSink
}

#[async_trait]
impl Run for Core {
    async fn run(&mut self) {
        while let Some(msg) = self.channel.recv().await {
            match msg {
                Message::ConnectionSuccess => {
                    self.sink.submit_command(CONNECTION_SUCCESS, true, Target::Auto).unwrap();
                    break;
                },
                Message::ConnectionError(e) => {
                    self.sink.submit_command(CONNECTION_ERROR, e, Target::Auto).unwrap();
                    return
                }
                Message::NetworkError(_) => unreachable!(),
                Message::NetworkCommand(_) => unreachable!()
            };
        }
        loop {
            if let Err(e) = self.buffer.try_submit(&mut self.channel, |v| v.into_result()) {
                self.sink.submit_command(NETWORK_ERROR, e, Target::Auto).unwrap();
                break;
            }
            if let Some(batch) = self.buffer.fast_forward() {
                self.sink.submit_command(NETWORK_COMMAND, batch, Target::Auto).unwrap();
            }
            if self.buffer.should_terminate() {
                break;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }
    }
}

struct Internal {
    ip: String,
    max_sub_buffer: Option<usize>
}

type Params = (String, Option<usize>);

impl super::base::Connection for Internal {
    type Message = Message;
    type Worker = Worker;
    type Core = Core;
    type Parameters = Params;

    fn max_messages(&self) -> usize {
        super::command::Buffer::new(self.max_sub_buffer).max_buffer()
    }

    fn new_worker(&self, channel: Sender<Self::Message>) -> Self::Worker {
        Worker::new(self.ip.clone(), channel)
    }

    fn new_core(&self, sink: ExtEventSink, channel: Receiver<Self::Message>) -> Self::Core {
        Core {
            sink,
            channel,
            buffer: super::command::Buffer::new(self.max_sub_buffer),
        }
    }

    fn new((ip, max_sub_buffer): Params) -> Option<Self> {
        Some(Internal {
            ip,
            max_sub_buffer
        })
    }
}

pub fn new(sink: ExtEventSink, params: Params) -> Option<BaseConnection> {
    BaseConnection::new::<Internal>(sink, params)
}
