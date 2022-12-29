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

use std::io::Result;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
    sync::oneshot,
    task::JoinHandle,
};

use crate::network_types as nt;
use crate::session::{Config, Session};
use crate::util::{broker_line, Type};

pub type ClientTaskResult = JoinHandle<Result<()>>;

const DEFAULT_NET_BUFFER_SIZE: usize = 1024;

async fn handle_connection(
    connection_string: &str,
    stop_signal: &mut oneshot::Receiver<()>,
) -> Result<TcpStream> {
    tokio::select! {
        res = TcpStream::connect(&connection_string) => res,
        _ = stop_signal => Err(std::io::Error::new(std::io::ErrorKind::Interrupted, "Connection aborted"))
    }
}

pub struct Client {
    stop_signal: Option<oneshot::Sender<()>>,
    index: usize,
    connection_string: String,
}

impl Client {
    pub fn new(
        connection_string: String,
        index: usize,
        config: Config,
    ) -> (Client, ClientTaskResult) {
        let (stop_signal, mut receiver) = oneshot::channel();
        broker_line(
            Type::ConnectionEvent,
            index,
            format!("Connecting with {}...", connection_string),
        );
        let motherfuckingrust = connection_string.clone();
        let task = tokio::spawn(async move {
            let stream = handle_connection(&connection_string, &mut receiver).await?;
            broker_line(
                Type::LogInfo,
                index,
                format!("Connected to {}", connection_string),
            );
            let mut task = ClientTask::new(stream, index, config);
            task.run(receiver).await
        });
        (
            Client {
                stop_signal: Some(stop_signal),
                index,
                connection_string: motherfuckingrust,
            },
            task,
        )
    }

    pub fn connection_string(&self) -> &str {
        &self.connection_string
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn stop(&mut self) {
        self.stop_signal.take().map(|v| v.send(()).unwrap());
    }
}

struct ClientTask {
    stream: BufReader<TcpStream>,
    //session: Option<Session>,
    client_index: usize,
    net_buffer: Vec<u8>,
    config: Config,
}

impl ClientTask {
    pub fn new(stream: TcpStream, client_index: usize, config: Config) -> ClientTask {
        ClientTask {
            stream: BufReader::new(stream),
            //session: None,
            client_index,
            net_buffer: vec![0; DEFAULT_NET_BUFFER_SIZE],
            config,
        }
    }

    pub fn kick<T>(msg: &str) -> Result<T> {
        Err(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("kicked, reason: {}", msg),
        ))
    }

    async fn handshake(&mut self) -> Result<Session> {
        let mut buffer: [u8; 40] = [0; 40]; //Hello is a 40 bytes packet.
        self.stream.read_exact(&mut buffer).await?;
        let packet = nt::Hello::from_bytes(buffer);
        match packet.matches(&nt::HELLO_PACKET) {
            nt::MatchResult::SignatureMismatch => Self::kick("wrong signature"),
            nt::MatchResult::VersionMismatch => Self::kick("wrong version"),
            nt::MatchResult::Ok => {
                let session = Session::new(self.client_index, self.config).await?;
                //self.session = Some(session);
                let hello = nt::HELLO_PACKET.to_bytes();
                self.stream.write_all(&hello).await?;
                Ok(session)
            }
        }
    }

    async fn read_command(&mut self) -> Result<nt::Command> {
        let len = self.stream.read_u32_le().await?;
        if len as usize > self.net_buffer.len() {
            self.net_buffer.resize(len as usize, 0);
        }
        self.stream
            .read_exact(&mut self.net_buffer[..len as usize])
            .await?;
        let cmd: nt::Command = match bincode::deserialize(&self.net_buffer[..len as usize]) {
            Ok(v) => v,
            Err(e) => return Self::kick(&e.to_string()),
        };
        Ok(cmd)
    }

    async fn main_loop(
        &mut self,
        session: &mut Session,
        mut stop_signal: oneshot::Receiver<()>,
    ) -> Result<()> {
        loop {
            tokio::select! {
                res = session.get_error() => res?,
                res = self.read_command() => if !session.handle_command(res?).await? {
                    break;
                },
                _ = &mut stop_signal => break
            }
        }
        Ok(())
    }

    pub async fn run(&mut self, mut stop_signal: oneshot::Receiver<()>) -> Result<()> {
        let mut session = tokio::select! {
            res = self.handshake() => res?,
            _ = &mut stop_signal => return Ok(())
        };
        let res = self.main_loop(&mut session, stop_signal).await;
        session.stop().await?;
        res
    }
}
