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

mod fd_map;
mod paths;

use std::collections::HashMap;
use std::io::Result;
use std::ops::Deref;
use std::path::{Path, PathBuf};

use tokio::io::AsyncWriteExt;

use crate::network_types as nt;

use self::fd_map::FdMap;
use self::paths::{Paths, Directory};

struct SpanData {
    message: Option<String>,
    value_set: Vec<(String, nt::Value)>,
    duration: f64,
}

pub struct Session {
    paths: Paths,
    fd_map: FdMap,
    spans: HashMap<nt::SpanId, SpanData>,
}

impl Session {
    pub async fn new(client_index: usize, max_fd_count: usize) -> Result<Session> {
        let paths = Paths::new(client_index).await?;
        Ok(Session {
            paths,
            fd_map: FdMap::new(max_fd_count),
            spans: HashMap::new(),
        })
    }

    //TODO: Figure out how to pass system informations
    pub async fn handle_command(&mut self, cmd: nt::Command) -> Result<()> {
        match cmd {
            nt::Command::SpanAlloc { id, metadata } => {
                let out = self.fd_map.open_file(&self.paths, id.id, Directory::Metadata).await?;
                let opt_file = format!("FILE={}\n", metadata.file.unwrap_or_default());
                let opt_name = format!("NAME={}\n", metadata.name);
                let opt_level = format!("LEVEL={}\n", metadata.level);
                let opt_line = match metadata.line {
                    Some(v) => format!("LINE={}\n", v),
                    None => "LINE=\n".into(),
                };
                let opt_target = format!("TARGET={}\n", metadata.target);
                let opt_mpath = format!("MODULE_PATH={}\n", metadata.module_path.unwrap_or_default());

                out.write_all(opt_file.as_bytes()).await?;
                out.write_all(opt_name.as_bytes()).await?;
                out.write_all(opt_level.as_bytes()).await?;
                out.write_all(opt_line.as_bytes()).await?;
                out.write_all(opt_target.as_bytes()).await?;
                out.write_all(opt_mpath.as_bytes()).await?;
                //TODO: Update span tree
                //TODO: Synchronize span data and tree with GUI sessions
            },
            nt::Command::SpanInit { span, parent, message, value_set } => {
                self.spans.insert(
                    span,
                    SpanData {
                        message,
                        value_set,
                        duration: 0.0,
                    },
                );
                //TODO: Update span tree
                //TODO: Synchronize span data and tree with GUI sessions
            },
            //TODO: Update span tree
            //TODO: Synchronize span data and tree with GUI sessions
            nt::Command::SpanFollows { span, follows } => todo!(),
            nt::Command::SpanValues { span, message, value_set } => {
                if let Some(span) = self.spans.get_mut(&span) {
                    if message.is_some() {
                        span.message = message;
                    }
                    for kv in value_set {
                        span.value_set.push(kv);
                    }
                }
                //TODO: Synchronize span data and tree with GUI sessions
            },
            //TODO: Synchronize span data and tree with GUI sessions
            //TODO: Write events file
            nt::Command::Event { span, metadata, time, message, value_set } => todo!(),
            //TODO: Synchronize span data and tree with GUI sessions
            nt::Command::SpanEnter(id) => todo!(),
            nt::Command::SpanExit { span, duration } => {
                if let Some(span) = self.spans.get_mut(&span) {
                    span.duration = duration;
                }
                //TODO: Synchronize span data and tree with GUI sessions
                //TODO: Write runs file
            },
            //TODO: Synchronize span data and tree with GUI sessions
            nt::Command::SpanFree(id) => todo!(),
            //TODO: Flush all opened file buffers
            //TODO: Write span tree file
            nt::Command::Terminate => todo!(),
        }
        Ok(())
    }
}
