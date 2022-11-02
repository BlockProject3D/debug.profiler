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

use tokio::{net::TcpStream, task::JoinHandle, io::AsyncReadExt, sync::oneshot::{Sender, channel}};
use std::io::Result;

pub type ClientTask = JoinHandle<(usize, Result<()>)>;

pub struct Client {
    stop_signal: Sender<()>,
    index: usize
}

impl Client {
    pub fn new(mut stream: TcpStream, addr: SocketAddr, index: usize) -> (Client, ClientTask) {
        let (stop_signal, mut receiver) = channel();
        let task = tokio::spawn(async move {
            let mut buffer: [u8; 512] = [0; 512];
            loop {
                tokio::select! {
                    res = stream.read(&mut buffer) => {
                        let len = res.unwrap();
                        if len <= 0 {
                            break;
                        }
                        println!("Read {} byte(s)", len);
                    },
                    _ = &mut receiver => break
                }
            }
            (index, Ok(()))
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
