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

use std::net::SocketAddr;

use std::io::Result;
use tokio::{
    io::{AsyncReadExt, BufReader, AsyncWriteExt},
    net::TcpStream,
    sync::oneshot::{channel, Receiver, Sender},
    task::JoinHandle,
};

use crate::{network_types as nt, session::Config};
use crate::session::Session;

pub type ClientTaskResult = JoinHandle<(usize, Result<()>)>;

const DEFAULT_NET_BUFFER_SIZE: usize = 1024;

pub struct Client {
    stop_signal: Sender<()>,
    index: usize,
}

impl Client {
    pub fn new(stream: TcpStream, addr: SocketAddr, index: usize) -> (Client, ClientTaskResult) {
        let (stop_signal, receiver) = channel();
        println!(
            "Client at address '{}' has been assigned index {}",
            addr, index
        );
        let task = tokio::spawn(async move {
            let mut task = ClientTask::new(stream, index);
            (index, task.run(receiver).await)
        });
        (Client { stop_signal, index }, task)
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn stop(self) {
        self.stop_signal.send(()).unwrap();
    }
}

struct ClientTask {
    stream: BufReader<TcpStream>,
    session: Option<Session>,
    client_index: usize,
    net_buffer: Vec<u8>
}

impl ClientTask {
    pub fn new(stream: TcpStream, client_index: usize) -> ClientTask {
        ClientTask {
            stream: BufReader::new(stream),
            session: None,
            client_index,
            net_buffer: vec![0; DEFAULT_NET_BUFFER_SIZE]
        }
    }

    pub fn kick(msg: &str) -> Result<()> {
        Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("kicked, reason: {}", msg),
        ))
    }

    pub async fn handle_read(&mut self) -> Result<()> {
        match &mut self.session {
            Some(v) => {
                let len = self.stream.read_u32_le().await?;
                if len as usize > self.net_buffer.len() {
                    self.net_buffer.resize(len as usize, 0);
                }
                self.stream.read_exact(&mut self.net_buffer[..len as usize]).await?;
                let cmd: nt::Command = match bincode::deserialize(&self.net_buffer[..len as usize]) {
                    Ok(v) => v,
                    Err(e) => return Self::kick(&e.to_string()),
                };
                v.handle_command(cmd).await?;
                Ok(())
            }
            None => {
                let mut buffer: [u8; 40] = [0; 40]; //Hello is a 40 bytes packet.
                self.stream.read_exact(&mut buffer).await?;
                let packet = nt::Hello::from_bytes(buffer);
                match packet.matches(&nt::HELLO_PACKET) {
                    nt::MatchResult::SignatureMismatch => Self::kick("wrong signature"),
                    nt::MatchResult::VersionMismatch => Self::kick("wrong version"),
                    nt::MatchResult::Ok => {
                        let session = Session::new(self.client_index, Config { max_fd_count: 2, inheritance: false }).await?;
                        self.session = Some(session);
                        let hello = nt::HELLO_PACKET.to_bytes();
                        self.stream.write_all(&hello).await?;
                        Ok(())
                    }
                }
            }
        }
    }

    pub async fn run(&mut self, mut stop_signal: Receiver<()>) -> Result<()> {
        loop {
            tokio::select! {
                res = self.handle_read() => res?,
                _ = &mut stop_signal => break
            }
        }
        Ok(())
    }
}
