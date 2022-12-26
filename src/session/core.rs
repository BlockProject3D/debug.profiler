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

use serde::Deserialize;
use std::io::Result;
use std::sync::Arc;
use std::time::Duration;
use chrono::{Local, NaiveDateTime, TimeZone};

use tokio::fs::File;
use tokio::io::{AsyncWriteExt, BufWriter};

use crate::network_types as nt;
use crate::network_types::Metadata;
use crate::session::file_manager::{FileManager, Span};
use crate::session::utils::csv_format_single;
use crate::util::{broker_line, Type};

use super::paths::Paths;
use super::state::{SpanInstance, SpanState};
use super::tree;
use super::utils::{csv_format, ValueSet};

const DEFAULT_MAX_FD_COUNT: usize = 2;
const DEFAULT_REFRESH_INTERVAL: u32 = 500; //500 ms

#[derive(Default, Debug, Copy, Clone, Deserialize)]
pub struct Config {
    pub max_fd_count: Option<usize>,
    pub inheritance: Option<bool>,
    pub refresh_interval: Option<u32>, //Refresh interval in ms
}

impl Config {
    pub fn get_max_fd_count(&self) -> usize {
        self.max_fd_count.unwrap_or(DEFAULT_MAX_FD_COUNT)
    }

    pub fn has_inheritance(&self) -> bool {
        self.inheritance.unwrap_or(true)
    }

    pub fn get_refresh_interval(&self) -> u32 {
        self.refresh_interval.unwrap_or(DEFAULT_REFRESH_INTERVAL)
    }
}

pub struct Session {
    paths: Paths,
    manager: FileManager,
    spans: SpanState,
    config: Config,
    tree: tree::SpanTree,
    client_index: usize,
}

fn duration_to_string(duration: &Duration) -> String {
    if duration.as_secs() > 0 {
        format!("{}s", duration.as_secs_f64())
    } else if duration.subsec_millis() > 0 {
        format!("{}ms", duration.subsec_millis())
    } else {
        format!("{}µs", duration.as_micros())
    }
}

impl Session {
    pub async fn new(client_index: usize, config: Config) -> Result<Session> {
        let paths = Paths::new(client_index).await?;
        Ok(Session {
            manager: FileManager::new(config.get_max_fd_count(), paths.clone()),
            paths,
            spans: SpanState::new(),
            config,
            tree: tree::SpanTree::new(),
            client_index,
        })
    }

    fn print_event(&self, id: u32, msg: String, value_set: ValueSet) {
        broker_line(
            Type::SpanEvent,
            self.client_index,
            format!(
                "{} {} {}",
                id,
                csv_format_single(msg, ' '),
                csv_format_single(value_set.to_string(), ' ')
            ),
        );
    }

    fn print_span(&self, id: u32, metadata: &Arc<Metadata>) {
        let file = metadata.file.as_deref().unwrap_or("None");
        let line = metadata
            .line
            .map(|v| v.to_string())
            .unwrap_or("None".into());
        let module = metadata.module_path.as_deref().unwrap_or("None");
        broker_line(
            Type::SpanAlloc,
            self.client_index,
            format!(
                "{} {} {} {} {} {} {}",
                id,
                csv_format_single(&metadata.name, ' '),
                metadata.level,
                csv_format_single(&metadata.target, ' '),
                csv_format_single(module, ' '),
                csv_format_single(file, ' '),
                line
            ),
        );
    }

    fn print_data_update(&mut self, id: u32) {
        if let Some(data) = self.spans.get_data_mut(id) {
            let now = std::time::Instant::now();
            let diff = now - data.last_update;
            if diff.as_millis() as u32 > self.config.get_refresh_interval() {
                data.last_update = now;
                let dropped = if data.is_dropped() { "D" } else { "L" };
                let active = if data.is_active() { "A" } else { "I" };
                let min = Duration::from(data.min);
                let max = Duration::from(data.max);
                let average = if data.run_count == 0 {
                    Duration::ZERO
                } else {
                    Duration::from(data.average) / data.run_count as u32
                };
                broker_line(
                    Type::SpanData,
                    self.client_index,
                    format!(
                        "{} {} {} {} {} {} {}",
                        id,
                        dropped,
                        active,
                        duration_to_string(&min),
                        duration_to_string(&max),
                        duration_to_string(&average),
                        data.run_count
                    ),
                );
            }
        }
    }

    fn print_path(&self, mut id: u32) {
        let whatever = id;
        let mut components = Vec::new();
        let path = self.spans.get_data(id).map(|v| v.metadata.name.as_ref()).unwrap_or("");
        components.push(path);
        while let Some(component) = self.tree.as_ref().find_parent(id) {
            let path = self.spans.get_data(component).map(|v| v.metadata.name.as_ref()).unwrap_or("root");
            components.push(path);
            id = component;
        }
        components.reverse();
        let str = components.join("/");
        broker_line(Type::SpanPath, self.client_index, format!("{} {}", whatever, str));
    }

    pub async fn get_error(&mut self) -> Result<()> {
        self.manager.get_error().await
    }

    pub async fn stop(self) -> Result<()> {
        self.manager.stop().await
    }

    pub async fn handle_command(&mut self, cmd: nt::Command) -> Result<bool> {
        match cmd {
            nt::Command::SpanAlloc { id, metadata } => {
                let metadata = Arc::new(metadata);
                self.manager.write_span(id.id, Span::Metadata(metadata.clone())).await;
                self.print_span(id.id, &metadata);
                self.tree
                    .add_node(tree::Span::with_metadata(id.id, metadata.clone()));
                self.spans.alloc_span(id.id, metadata);
                self.print_path(id.id);
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
                        duration: Duration::new(0, 0),
                    },
                );
                if let Some(parent) = parent {
                    if self.tree.relocate_node(span.id, parent.id) {
                        self.print_path(span.id);
                    }
                }
            }
            nt::Command::SpanFollows { span, follows } => {
                if let Some(parent) = self.tree.as_ref().find_parent(follows.id) {
                    if self.tree.relocate_node(span.id, parent) {
                        self.print_path(span.id);
                    }
                }
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
            }
            nt::Command::Event {
                span,
                metadata,
                time,
                message,
                value_set,
            } => {
                let date = NaiveDateTime::from_timestamp_millis(time)
                    .map(|v| Local.from_utc_datetime(&v))
                    .unwrap_or(Local::now());
                let date_str = date.format("%a %b %d &Y %I:%M:%S %P");
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
                if self.config.has_inheritance() {
                    if let Some(data) = self.spans.get_data(span.id) {
                        if let Some(instance) = self.spans.get_instance(&span) {
                            value_set.inherit_from(
                                &data.metadata.name,
                                instance.value_set.clone().into_iter(),
                            )
                        }
                    }
                }
                self.manager.write_span(span.id, Span::Event(span.instance, msg.clone(), value_set.clone())).await;
                self.print_event(span.id, msg, value_set);
            }
            nt::Command::SpanEnter(id) => {
                if let Some(span) = self.spans.get_instance_mut(&id) {
                    span.active = true;
                }
                self.print_data_update(id.id);
            }
            nt::Command::SpanExit { span, duration } => {
                if let Some(data) = self.spans.get_instance_mut(&span) {
                    data.active = false;
                    data.duration = Duration::new(duration.seconds.into(), duration.nano_seconds);
                }
                self.print_data_update(span.id);
            }
            nt::Command::SpanFree(id) => {
                if let Some(mut data) = self.spans.free_instance(&id) {
                    if self.config.has_inheritance() {
                        if let Some(parent) = self.tree.as_ref().find_parent(id.id) {
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
                    self.manager.write_span(id.id, Span::Run(id.instance, data)).await;
                }
                self.print_data_update(id.id);
            }
            nt::Command::Terminate => {
                let file = File::create(self.paths.get_root().join("times.csv")).await?;
                let mut buffer = BufWriter::new(file);
                for (index, data) in self.spans.iter_mut() {
                    data.average /= data.run_count as u32;
                    buffer
                        .write_all(
                            (csv_format([
                                &*index.to_string(),
                                &data.min.as_secs().to_string(),
                                &data.min.subsec_millis().to_string(),
                                &(data.min.subsec_micros() - (data.min.subsec_millis() * 1000))
                                    .to_string(),
                                &data.max.as_secs().to_string(),
                                &data.max.subsec_millis().to_string(),
                                &(data.max.subsec_micros() - (data.max.subsec_millis() * 1000))
                                    .to_string(),
                                &data.average.as_secs().to_string(),
                                &data.average.subsec_millis().to_string(),
                                &(data.average.subsec_micros()
                                    - (data.average.subsec_millis() * 1000))
                                    .to_string(),
                            ]) + "\n")
                                .as_bytes(),
                        )
                        .await?;
                }
                buffer.flush().await?;
                let file = File::create(self.paths.get_root().join("tree.txt")).await?;
                let mut buffer = BufWriter::new(file);
                self.tree.write(&mut buffer).await?;
                buffer.flush().await?;
                return Ok(false);
            }
            nt::Command::Project(project) => {
                self.manager.write_project(project);
            }
        }
        Ok(true)
    }
}
