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
    task::JoinHandle, sync::mpsc::{Sender, channel, Receiver}, net::TcpStream
};

use crate::client_manager::{ClientManager, JoinResult};

const DEFAULT_PORT: u16 = 4026;
const COMMAND_CHAN_SIZE: usize = 4;

#[derive(Debug)]
pub enum Command {
    Stop,
    Connect(String)
}

pub struct Server {
    command_sender: Sender<Command>,
    task: JoinHandle<Result<()>>,
}

impl Server {
    pub async fn new() -> Result<Server> {
        let (command_sender, receiver) = channel(COMMAND_CHAN_SIZE);
        let task = tokio::spawn(async {
            let mut task = ServerTask::new(receiver);
            task.run().await?;
            task.stop().await;
            Ok(())
        });
        Ok(Server { command_sender, task })
    }

    pub async fn connect(&mut self, ip: &str) {
        self.command_sender.send(Command::Connect(ip.into())).await.unwrap()
    }

    pub async fn stop(self) {
        self.command_sender.send(Command::Stop).await.unwrap();
        if let Err(e) = self.task.await {
            eprintln!("An error has occurred while running the server: {}", e);
        }
    }
}

struct ServerTask {
    manager: ClientManager,
    command_receiver: Receiver<Command>,
}

impl ServerTask {
    pub fn new(command_receiver: Receiver<Command>) -> ServerTask {
        ServerTask {
            manager: ClientManager::new(),
            command_receiver,
        }
    }

    fn handle_client_stop(&mut self, res: JoinResult<(usize, Result<()>)>) {
        match res {
            Ok((index, res)) => {
                self.manager.remove(index);
                match res {
                    Ok(_) => println!("Client with index {} has disconnected", index),
                    Err(e) => eprintln!("Client with index {} has errored: {}", index, e),
                }
            }
            Err(e) => {
                eprintln!("A client has panicked: {}", e);
            }
        }
    }

    async fn handle_command(&mut self, cmd: Command) -> Result<bool> {
        match cmd {
            Command::Stop => Ok(true),
            Command::Connect(mut v) => {
                if !v.contains(':') {
                    v += &format!(":{}", DEFAULT_PORT);
                }
                let stream = match TcpStream::connect(&v).await {
                    Err(e) => {
                        eprintln!("Failed to connect to application at {}: {}", v, e);
                        return Ok(false);
                    }
                    Ok(v) => v
                };
                let addr = stream.peer_addr()?;
                self.manager.add(stream, addr);
                Ok(false)
            }
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        loop {
            tokio::select! {
                res = self.command_receiver.recv() => {
                    if let Some(cmd) = res {
                        let flag = self.handle_command(cmd).await?;
                        if flag {
                            break;
                        }
                    }
                },
                res = self.manager.get_client_stop() => {
                    self.handle_client_stop(res);
                }
            }
        }
        Ok(())
    }

    pub async fn stop(self) {
        self.manager.stop().await
    }
}
