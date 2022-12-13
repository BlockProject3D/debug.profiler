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

use std::error::Error;
use crate::server::client_manager::ClientManager;
use super::DEFAULT_PORT;

#[derive(Debug)]
pub enum Command {
    Stop,
    Connect(String),
}

pub enum CommandResult {
    Continue,
    Terminate,
    Error(String)
}

impl<T: Error> From<T> for CommandResult {
    fn from(e: T) -> Self {
        CommandResult::Error(e.to_string())
    }
}

pub struct CommandHandler {
}

impl CommandHandler {
    pub fn new() -> CommandHandler {
        CommandHandler {}
    }

    pub async fn handle_command(&mut self, clients: &mut ClientManager, cmd: Command) -> CommandResult {
        match cmd {
            Command::Stop => CommandResult::Terminate,
            Command::Connect(mut v) => {
                if !v.contains(':') {
                    v += &format!(":{}", DEFAULT_PORT);
                }
                clients.add(v);
                CommandResult::Continue
            }
        }
    }
}
