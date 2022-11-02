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

use tokio::{net::TcpListener, sync::oneshot::{Sender, channel, Receiver}, task::JoinHandle};
use std::io::Result;

use crate::client_manager::{ClientManager, JoinResult};

pub struct Server {
    stop_signal: Sender<()>,
    task: JoinHandle<Result<()>>
}

impl Server {
    pub async fn new(ip: &str) -> Result<Server> {
        let (stop_signal, receiver) = channel();
        let socket = TcpListener::bind(ip).await?;
        let task = tokio::spawn(async {
            let mut task = ServerTask::new(socket, receiver);
            task.run().await?;
            task.stop().await;
            Ok(())
        });
        Ok(Server {
            stop_signal,
            task
        })
    }

    pub async fn stop(self) {
        self.stop_signal.send(()).unwrap();
        if let Err(e) = self.task.await {
            eprintln!("An error has occurred while running the server: {}", e);
        }
    }
}

struct ServerTask {
    manager: ClientManager,
    socket: TcpListener,
    stop_signal: Receiver<()>
}

impl ServerTask {
    pub fn new(socket: TcpListener, stop_signal: Receiver<()>) -> ServerTask {
        ServerTask {
            manager: ClientManager::new(),
            socket,
            stop_signal
        }
    }

    fn handle_client_stop(&mut self, res: JoinResult<(usize, Result<()>)>) {
        match res {
            Ok((index, res)) => {
                self.manager.remove(index);
                match res {
                    Ok(_) => println!("Client with index {} has disconnected", index),
                    Err(e) => eprintln!("Client with index {} has errored: {}", index, e)
                }
            },
            Err(e) => {
                eprintln!("A client has panicked: {}", e);
            }
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        loop {
            tokio::select! {
                res = self.socket.accept() => {
                    let (stream, addr) = res?;
                    println!("Client has connected: {}", addr);
                    self.manager.add(stream, addr);
                },
                res = self.manager.get_client_stop() => {
                    self.handle_client_stop(res);
                },
                _ = &mut self.stop_signal => break
            }
        }
        Ok(())
    }

    pub async fn stop(self) {
        self.manager.stop().await
    }
}
