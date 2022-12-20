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

use super::DEFAULT_PORT;
use crate::server::client_manager::ClientManager;
use crate::session::Config;
use crate::util::{broker_line, Type};
use serde::Deserialize;
use std::fmt::Display;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Command {
    Stop,
    Connect(String),
    Kick(usize),
    List,
    Config(Config),
}

#[derive(Eq, PartialEq)]
pub enum Event {
    Continue,
    Terminate,
}

#[derive(Debug)]
pub struct Error(String);

impl AsRef<str> for Error {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl<T: Display> From<T> for Error {
    fn from(e: T) -> Self {
        Error(e.to_string())
    }
}

pub struct CommandHandler {}

impl CommandHandler {
    pub fn new() -> CommandHandler {
        CommandHandler {}
    }

    pub async fn handle_command(
        &mut self,
        clients: &mut ClientManager,
        cmd: Command,
    ) -> Result<Event, Error> {
        match cmd {
            Command::Stop => Ok(Event::Terminate),
            Command::Connect(mut v) => {
                if !v.contains(':') {
                    v += &format!(":{}", DEFAULT_PORT);
                }
                clients.add(v);
                Ok(Event::Continue)
            }
            Command::Kick(client) => {
                let client = clients.get(client).ok_or("Client does not exist")?;
                client.stop();
                Ok(Event::Continue)
            }
            Command::List => {
                for v in clients.iter() {
                    broker_line(Type::LogInfo, v.index(), v.connection_string());
                }
                Ok(Event::Continue)
            }
            Command::Config(cfg) => {
                clients.set_config(cfg);
                Ok(Event::Continue)
            }
        }
    }
}
