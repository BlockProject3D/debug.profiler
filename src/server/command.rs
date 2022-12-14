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

use std::fmt::Display;
use std::time::Duration;
use serde::Deserialize;
use crate::server::client_manager::ClientManager;
use crate::util::{broker_line, csv_format_single, Level};
use super::DEFAULT_PORT;

#[derive(Debug, Deserialize)]
pub enum Command {
    Stop,
    Connect(String),
    Kick(usize),
    List,
    Spans(usize),
    Metadata(usize)
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

// Annoyingly long to write the same kind of minimalistic if block so hijack ternaries in Rust.
macro_rules! ter {
    (($value: expr) ? ($yes: expr) : ($no: expr) ) => {
        if $value {
            $yes
        } else {
            $no
        }
    };
}

fn duration_to_string(d: &Duration) -> String {
    if d.as_secs() > 0 {
        format!("{}s", d.as_secs_f64())
    } else if d.subsec_millis() > 0 {
        format!("{}ms", d.as_millis())
    } else {
        format!("{}µs", d.as_micros())
    }
}

pub struct CommandHandler {
}

impl CommandHandler {
    pub fn new() -> CommandHandler {
        CommandHandler {}
    }

    pub async fn handle_command(&mut self, clients: &mut ClientManager, cmd: Command) -> Result<Event, Error> {
        match cmd {
            Command::Stop => Ok(Event::Terminate),
            Command::Connect(mut v) => {
                if !v.contains(':') {
                    v += &format!(":{}", DEFAULT_PORT);
                }
                clients.add(v);
                Ok(Event::Continue)
            },
            Command::Kick(client) => {
                let client = clients.get(client).ok_or("Client does not exist")?;
                client.stop();
                Ok(Event::Continue)
            },
            Command::List => {
                for v in clients.iter() {
                    broker_line(Level::Info, v.index(), v.connection_string());
                }
                Ok(Event::Continue)
            },
            Command::Spans(client1) => {
                let client = clients.get(client1).ok_or("Client does not exist")?;
                for v in client.spans().get_iter() {
                    let dropped = ter!((v.is_dropped.get()) ? ("D") : ("L")); //D = dead, L = live
                    let active = ter!((v.is_active.get()) ? ("A") : ("I")); //A = active, I = inactive
                    let min = Duration::from(v.min.get());
                    let max = Duration::from(v.max.get());
                    let average = Duration::from(v.average.get());
                    broker_line(Level::Info, client1,
                                format!("{} {} {} {} {} {} {}",
                                        v.id, dropped, active,
                                        duration_to_string(&min),
                                        duration_to_string(&max),
                                        duration_to_string(&average),
                                        v.run_count.get()
                                ));
                }
                Ok(Event::Continue)
            },
            Command::Metadata(client1) => {
                let client = clients.get(client1).ok_or("Client does not exist")?;
                for v in client.spans().get_iter() {
                    let file = v.metadata.file.as_deref().unwrap_or("None");
                    let line = v.metadata.line.map(|v| v.to_string()).unwrap_or("None".into());
                    let module = v.metadata.module_path.as_deref().unwrap_or("None");
                    broker_line(Level::Info, client1,
                                format!("{} {} {} {} {} {} {}",
                                    v.id,
                                    csv_format_single(&v.metadata.name, ' '),
                                    v.metadata.level,
                                    csv_format_single(&v.metadata.target, ' '),
                                    csv_format_single(module, ' '),
                                    csv_format_single(file, ' '),
                                    line
                                ));
                }
                Ok(Event::Continue)
            }
        }
    }
}
