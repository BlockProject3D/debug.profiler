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
use crate::session::utils::csv_format_single;
use crate::util::{broker_line, Type};

use super::fd_map::FdMap;
use super::paths::{Directory, Paths};
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

/*pub struct Config {
    pub max_fd_count: usize,
    pub inheritance: bool,
    pub refresh_interval: u32 //Refresh interval in ms
}*/

pub struct Session {
    paths: Paths,
    fd_map: FdMap,
    spans: SpanState,
    config: Config,
    tree: tree::Span,
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
            paths,
            fd_map: FdMap::new(config.get_max_fd_count()),
            spans: SpanState::new(),
            config,
            tree: tree::Span::new(),
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
                let average = Duration::from(data.average);
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
                self.print_span(id.id, &metadata);
                self.tree
                    .add_node(tree::Span::with_metadata(id.id, metadata.clone()));
                self.spans.alloc_span(id.id, metadata);
                //TODO: Synchronize tree with GUI sessions
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
                    self.tree.relocate_node(span.id, parent.id);
                }
                //TODO: Synchronize tree with GUI sessions
            }
            nt::Command::SpanFollows { span, follows } => {
                if let Some(parent) = self.tree.find_parent(follows.id) {
                    self.tree.relocate_node(span.id, parent);
                }
                //TODO: Synchronize tree with GUI sessions
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
                let out = self
                    .fd_map
                    .open_file(&self.paths, span.id, Directory::Events)
                    .await?;
                out.write_all(
                    (csv_format([&*span.instance.to_string(), &msg])
                        + ","
                        + &value_set.clone().to_string()
                        + "\n")
                        .as_bytes(),
                )
                .await?;
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
                            &data.duration.as_secs().to_string(),
                            &data.duration.subsec_millis().to_string(),
                            &(data.duration.subsec_micros()
                                - (data.duration.subsec_millis() * 1000))
                                .to_string(),
                        ]) + ","
                            + &data.value_set.clone().to_string()
                            + "\n")
                            .as_bytes(),
                    )
                    .await?;
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
                self.fd_map.flush().await?;
            }
            nt::Command::Project {
                app_name,
                name,
                version,
                target,
                command_line,
                cpu,
            } => {
                let file = File::create(self.paths.get_root().join("info.csv")).await?;
                let mut buffer = BufWriter::new(file);
                buffer
                    .write_all((csv_format(["AppName", &app_name]) + "\n").as_bytes())
                    .await?;
                buffer
                    .write_all((csv_format(["Name", &name]) + "\n").as_bytes())
                    .await?;
                buffer
                    .write_all((csv_format(["Version", &version]) + "\n").as_bytes())
                    .await?;
                buffer
                    .write_all((csv_format(["CommandLine", &command_line]) + "\n").as_bytes())
                    .await?;
                buffer
                    .write_all((csv_format(["TargetOs", &target.os]) + "\n").as_bytes())
                    .await?;
                buffer
                    .write_all((csv_format(["TargetFamily", &target.family]) + "\n").as_bytes())
                    .await?;
                buffer
                    .write_all((csv_format(["TargetArch", &target.arch]) + "\n").as_bytes())
                    .await?;
                if let Some(cpu) = cpu {
                    buffer
                        .write_all((csv_format(["CpuName", &cpu.name]) + "\n").as_bytes())
                        .await?;
                    buffer
                        .write_all(
                            (csv_format(["CpuCoreCount", &cpu.core_count.to_string()]) + "\n")
                                .as_bytes(),
                        )
                        .await?;
                }
                buffer.flush().await?;
            }
        }
        Ok(())
    }
}
