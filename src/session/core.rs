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
use std::sync::Arc;

use time::macros::format_description;
use time::OffsetDateTime;
use time_tz::OffsetDateTimeExt;
use tokio::fs::File;
use tokio::io::{AsyncWriteExt, BufWriter};

use crate::network_types as nt;

use super::fd_map::FdMap;
use super::paths::{Directory, Paths};
use super::state::{SpanInstance, SpanState};
use super::tree;
use super::utils::{csv_format, ValueSet};

pub struct Config {
    pub max_fd_count: usize,
    pub inheritance: bool,
}

pub struct Session {
    paths: Paths,
    fd_map: FdMap,
    spans: SpanState,
    config: Config,
    tree: tree::Span,
}

impl Session {
    pub async fn new(client_index: usize, config: Config) -> Result<Session> {
        let paths = Paths::new(client_index).await?;
        Ok(Session {
            paths,
            fd_map: FdMap::new(config.max_fd_count),
            spans: SpanState::new(),
            config,
            tree: tree::Span::new(),
        })
    }

    //TODO: Figure out how to pass system informations
    pub async fn handle_command(&mut self, cmd: nt::Command) -> Result<()> {
        match cmd {
            nt::Command::SpanAlloc { id, metadata } => {
                let out = self
                    .fd_map
                    .open_file(&self.paths, id.id, Directory::Metadata)
                    .await?;
                let metadata = Arc::new(metadata);
                let opt_file = format!("File,{}\n", metadata.file.as_deref().unwrap_or_default());
                let opt_name = format!("Name,{}\n", metadata.name);
                let opt_level = format!("Level,{}\n", metadata.level);
                let opt_line = match metadata.line {
                    Some(v) => format!("Line,{}\n", v),
                    None => "Line,\n".into(),
                };
                let opt_target = format!("Target,{}\n", metadata.target);
                let opt_mpath = format!(
                    "Module path,{}\n",
                    metadata.module_path.as_deref().unwrap_or_default()
                );

                out.write_all(opt_file.as_bytes()).await?;
                out.write_all(opt_name.as_bytes()).await?;
                out.write_all(opt_level.as_bytes()).await?;
                out.write_all(opt_line.as_bytes()).await?;
                out.write_all(opt_target.as_bytes()).await?;
                out.write_all(opt_mpath.as_bytes()).await?;
                self.tree
                    .add_node(tree::Span::with_metadata(id.id, metadata.clone()));
                self.spans.alloc_span(id.id, metadata);
                //TODO: Synchronize span data and tree with GUI sessions
            }
            nt::Command::SpanInit {
                span,
                parent,
                message,
                value_set,
            } => {
                self.spans.alloc_instance(
                    &span,
                    SpanInstance {
                        message,
                        active: false,
                        value_set: value_set.into(),
                        duration: 0.0,
                    },
                );
                if let Some(parent) = parent {
                    self.tree.relocate_node(span.id, parent.id);
                }
                //TODO: Synchronize span data and tree with GUI sessions
            }
            nt::Command::SpanFollows { span, follows } => {
                if let Some(parent) = self.tree.find_parent(follows.id) {
                    self.tree.relocate_node(span.id, parent);
                }
                //TODO: Synchronize span data and tree with GUI sessions
            }
            nt::Command::SpanValues {
                span,
                message,
                value_set,
            } => {
                if let Some(span) = self.spans.get_instance_mut(&span) {
                    if message.is_some() {
                        span.message = message;
                    }
                    span.value_set.extend(value_set);
                }
                //TODO: Synchronize span data and tree with GUI sessions
            }
            nt::Command::Event {
                span,
                metadata,
                time,
                message,
                value_set,
            } => {
                let date = OffsetDateTime::from_unix_timestamp(time)
                    .unwrap_or(OffsetDateTime::now_utc())
                    .to_timezone(
                        time_tz::system::get_timezone()
                            .unwrap_or(time_tz::timezones::db::us::CENTRAL),
                    );
                let date_format = format_description!("[weekday repr:short] [month repr:short] [day] [hour repr:12]:[minute]:[second] [period case:upper]");
                let date_str = date.format(date_format).unwrap_or("<error>".into());
                let (target, module) = metadata.get_target_module();
                let msg = format!(
                    "({}) <{}> {}: {}",
                    date_str,
                    target,
                    module.unwrap_or("main"),
                    message.as_ref().unwrap_or(&metadata.name)
                );
                let span = span.unwrap_or(nt::SpanId { id: 0, instance: 0 });
                let mut value_set = ValueSet::from(value_set);
                if self.config.inheritance {
                    if let Some(data) = self.spans.get_data(span.id) {
                        if let Some(instance) = self.spans.get_instance(&span) {
                            value_set.inherit_from(
                                &data.metadata.name,
                                instance.value_set.clone().into_iter(),
                            )
                        }
                    }
                }
                let out = self
                    .fd_map
                    .open_file(&self.paths, span.id, Directory::Events)
                    .await?;
                out.write_all(
                    (csv_format([&*span.instance.to_string(), &msg])
                        + ","
                        + &value_set.to_string()
                        + "\n")
                        .as_bytes(),
                )
                .await?;
                //TODO: Synchronize span data and tree with GUI sessions
            }
            nt::Command::SpanEnter(id) => {
                if let Some(span) = self.spans.get_instance_mut(&id) {
                    span.active = true;
                }
                //TODO: Synchronize span data and tree with GUI sessions
            }
            nt::Command::SpanExit { span, duration } => {
                if let Some(data) = self.spans.get_instance_mut(&span) {
                    data.active = false;
                    data.duration = duration;
                }
                //TODO: Synchronize span data and tree with GUI sessions
            }
            nt::Command::SpanFree(id) => {
                if let Some(mut data) = self.spans.free_instance(&id) {
                    if self.config.inheritance {
                        if let Some(parent) = self.tree.find_parent(id.id) {
                            if let Some(data1) = self.spans.get_data(parent) {
                                if let Some(parent) = self.spans.get_any_instance(parent) {
                                    data.value_set.inherit_from(
                                        &data1.metadata.name,
                                        parent.value_set.clone().into_iter(),
                                    );
                                }
                            }
                        }
                    }
                    let out = self
                        .fd_map
                        .open_file(&self.paths, id.id, Directory::Runs)
                        .await?;
                    out.write_all(
                        (csv_format([
                            &*id.instance.to_string(),
                            &data.message.as_deref().unwrap_or_default(),
                            &data.duration.to_string(),
                        ]) + ","
                            + &data.value_set.clone().to_string()
                            + "\n")
                            .as_bytes(),
                    )
                    .await?;
                }
                //TODO: Synchronize span data and tree with GUI sessions
            }
            nt::Command::Terminate => {
                let file = File::create(self.paths.get_root().join("tree.txt")).await?;
                let mut buffer = BufWriter::new(file);
                self.tree.write(&mut buffer).await?;
                buffer.flush().await?;
                self.fd_map.flush().await?;
                //TODO: Synchronize with GUI sessions
            }
        }
        Ok(())
    }
}
