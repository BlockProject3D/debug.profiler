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

use server::Server;
use tokio::io::{BufReader, AsyncBufReadExt};
use std::io::Result;

mod client;
mod client_manager;
mod network_types;
mod server;
mod session;

async fn read_command(server: &mut Server) -> Result<()> {
    let mut buffer = BufReader::new(tokio::io::stdin());
    let mut str = String::default();
    while buffer.read_line(&mut str).await? > 0 {
        println!("Command string: {}", str);
        //Hack for tokio defect: the read_line function reads too many characters!
        if str.chars().last().unwrap() == '\n' {
            str.remove(str.len() - 1);
        }
        if str.chars().last().unwrap() == '\r' {
            str.remove(str.len() - 1);
        }
        //End

        if str == "exit" {
            break;
        }
        let mut split = str.split(" ");
        if let Some(cmd) = split.next() {
            if let Some(arg) = split.next() {
                if cmd == "connect" {
                    server.connect(arg).await;
                }
            }
        }
        str = String::default(); //Hack for tokio defect: somehow memory contains garbage
    }
    Ok(())
}

async fn run() {
    let server = Server::new().await;
    match server {
        Ok(mut v) => {
            if let Err(e) = read_command(&mut v).await {
                eprintln!("Failed to read standard input: {}", e);
            }
            v.stop().await;
        }
        Err(e) => eprintln!("Failed to start server: {}", e),
    }
}

#[tokio::main]
async fn main() {
    run().await
}
