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

use tokio::sync::mpsc::{UnboundedReceiver};
use crate::thread::base::ConnectionWrapper;
//use crate::thread::auto_discover::AutoDiscoveryConnection;
use crate::thread::Command;
use crate::thread::connection;

pub struct BackgroundThread {
    channel: UnboundedReceiver<Command>,
    connection: ConnectionWrapper,
    //auto_discovery: Option<AutoDiscoveryConnection>
}

impl BackgroundThread {
    pub fn new(channel: UnboundedReceiver<Command>) -> BackgroundThread {
        BackgroundThread {
            channel,
            connection: ConnectionWrapper::none(),
            //auto_discovery: None
        }
    }

    pub fn run(&mut self) {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_io()
            .enable_time()
            .build().unwrap();
        runtime.block_on(async {
            while let Some(cmd) = self.channel.recv().await {
                match cmd {
                    Command::Connect { ip, sink, max_sub_buffer } => {
                        self.connection.end().await;
                        self.connection = ConnectionWrapper::new(connection::new, sink, (ip, max_sub_buffer));
                        if self.connection.is_some() {
                            //If we have a new connection, terminate auto-discovery service.
                            //self.auto_discovery.take().map(|v| v.end());
                        }
                    },
                    Command::Terminate => break,
                    Command::Disconnect => { // Can't be inline because rust cannot accept
                        // Option<()> as ().
                        self.connection.end().await
                    }
                    Command::StartAutoDiscovery(sink) => {
                        /*if self.connection.is_some() {
                            //If we are already connected skip...
                            continue;
                        }*/
                        //self.auto_discovery = AutoDiscoveryConnection::new(sink, ());
                    }
                }
            }
            self.connection.end().await
        });
        /*loop {
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
                    Command::Connect { ip, sink, max_sub_buffer } => {
                        if self.connection.is_some() {
                            continue;
                        }
                        self.connection = Connection::new(sink, (ip, max_sub_buffer));
                        if self.connection.is_some() {
                            //If we have a new connection, terminate auto-discovery service.
                            //self.auto_discovery.take().map(|v| v.end());
                        }
                    },
                    Command::Terminate => break,
                    Command::Disconnect => { // Can't be inline because rust cannot accept
                        // Option<()> as ().
                        self.connection.take().map(|v| v.end());
                    }
                    Command::StartAutoDiscovery(sink) => {
                        if self.connection.is_some() {
                            //If we are already connected skip...
                            continue;
                        }
                        //self.auto_discovery = AutoDiscoveryConnection::new(sink, ());
                    }
                }
            }
            //Update connections.
            self.connection = self.connection.take().and_then(|v| v.step());
            //self.auto_discovery = self.auto_discovery.take().and_then(|v| v.step());
            std::thread::sleep(Duration::from_millis(50));
        }
        //Terminate all connections.
        self.connection.take().map(|v| v.end());
        //self.auto_discovery.take().map(|v| v.end());*/
    }
}
