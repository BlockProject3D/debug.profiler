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

use crate::network_types as nt;
use crate::session::fd_map::FdMap;
use crate::session::paths::{Directory, Paths};
use crate::session::state::SpanInstance;
use crate::session::utils::{csv_format, ValueSet};
use std::io::Result;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs::File;
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio::sync::{mpsc, oneshot};
use tokio::task::JoinHandle;

const MAX_CHANNEL_SIZE: usize = 512;

pub enum Span {
    Metadata(Arc<nt::Metadata>),
    Event(u32, String, ValueSet),
    Run(u32, SpanInstance),
}

pub struct FileManager {
    channel: mpsc::Sender<(u32, Span)>,
    error_channel_out: mpsc::Receiver<std::io::Error>,
    error_channel_in: mpsc::Sender<std::io::Error>,
    kill_channel: oneshot::Sender<()>,
    handle: JoinHandle<()>,
    project_handle: Option<JoinHandle<()>>,
    paths: Paths,
}

impl FileManager {
    pub fn new(max_fd_count: usize, paths: Paths) -> FileManager {
        let (channel_in, channel_out) = mpsc::channel(MAX_CHANNEL_SIZE);
        let (error_channel_in, error_channel_out) = mpsc::channel(MAX_CHANNEL_SIZE);
        let (kill_channel_in, kill_channel_out) = oneshot::channel();
        let motherfuckingrust = error_channel_in.clone();
        let motherfuckingrust2 = paths.clone();
        let handle = tokio::spawn(async move {
            let task = FileManagerTask {
                fd_map: FdMap::new(max_fd_count),
                paths: motherfuckingrust2,
                channel: channel_out,
                kill_channel: kill_channel_out,
                error_channel: motherfuckingrust,
            };
            task.run().await
        });
        FileManager {
            channel: channel_in,
            error_channel_out,
            error_channel_in,
            kill_channel: kill_channel_in,
            project_handle: None,
            handle,
            paths,
        }
    }

    pub async fn write_span(&mut self, id: u32, span: Span) {
        //Using map_err to circumvent Rust Debug requirement
        self.channel.send((id, span)).await.map_err(|_| ()).unwrap();
    }

    pub fn write_project(&mut self, project: nt::Project) {
        let error_channel = self.error_channel_in.clone();
        let motherfuckingrust = self.paths.get_root().join("info.csv");
        self.project_handle = tokio::spawn(async move {
            if let Err(e) = write_project_internal(motherfuckingrust, project).await {
                error_channel.send(e).await.unwrap();
            }
        })
        .into();
    }

    pub async fn get_error(&mut self) -> Result<()> {
        self.error_channel_out
            .recv()
            .await
            .map(|e| Err(e))
            .unwrap_or(Ok(()))
    }

    pub async fn stop(mut self) -> Result<()> {
        if let Some(v) = self.project_handle {
            v.await.unwrap();
        }
        if let Ok(e) = self.error_channel_out.try_recv() {
            return Err(e);
        }
        self.kill_channel.send(()).unwrap();
        self.handle.await.unwrap();
        if let Ok(e) = self.error_channel_out.try_recv() {
            Err(e)
        } else {
            Ok(())
        }
    }
}

async fn write_project_internal(path: PathBuf, project: nt::Project) -> Result<()> {
    let file = File::create(path).await?;
    let mut buffer = BufWriter::new(file);
    buffer
        .write_all((csv_format(["AppName", &project.app_name]) + "\n").as_bytes())
        .await?;
    buffer
        .write_all((csv_format(["Name", &project.name]) + "\n").as_bytes())
        .await?;
    buffer
        .write_all((csv_format(["Version", &project.version]) + "\n").as_bytes())
        .await?;
    buffer
        .write_all((csv_format(["CommandLine", &project.command_line]) + "\n").as_bytes())
        .await?;
    buffer
        .write_all((csv_format(["TargetOs", &project.target.os]) + "\n").as_bytes())
        .await?;
    buffer
        .write_all((csv_format(["TargetFamily", &project.target.family]) + "\n").as_bytes())
        .await?;
    buffer
        .write_all((csv_format(["TargetArch", &project.target.arch]) + "\n").as_bytes())
        .await?;
    if let Some(cpu) = project.cpu {
        buffer
            .write_all((csv_format(["CpuName", &cpu.name]) + "\n").as_bytes())
            .await?;
        buffer
            .write_all(
                (csv_format(["CpuCoreCount", &cpu.core_count.to_string()]) + "\n").as_bytes(),
            )
            .await?;
    }
    buffer.flush().await
}

struct FileManagerTask {
    fd_map: FdMap,
    paths: Paths,
    channel: mpsc::Receiver<(u32, Span)>,
    error_channel: mpsc::Sender<std::io::Error>,
    kill_channel: oneshot::Receiver<()>,
}

impl FileManagerTask {
    async fn handle_command_internal(&mut self, span_id: u32, span: Span) -> Result<()> {
        match span {
            Span::Metadata(metadata) => {
                let out = self
                    .fd_map
                    .open_file(&self.paths, span_id, Directory::Metadata)
                    .await?;
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
            }
            Span::Event(instance_id, msg, value_set) => {
                let out = self
                    .fd_map
                    .open_file(&self.paths, span_id, Directory::Events)
                    .await?;
                out.write_all(
                    (csv_format([&*instance_id.to_string(), &msg])
                        + ","
                        + &value_set.clone().to_string()
                        + "\n")
                        .as_bytes(),
                )
                .await?;
            }
            Span::Run(instance_id, instance) => {
                let out = self
                    .fd_map
                    .open_file(&self.paths, span_id, Directory::Runs)
                    .await?;
                out.write_all(
                    (csv_format([
                        &*instance_id.to_string(),
                        &instance.message.as_deref().unwrap_or_default(),
                        &instance.duration.as_secs().to_string(),
                        &instance.duration.subsec_millis().to_string(),
                        &(instance.duration.subsec_micros()
                            - (instance.duration.subsec_millis() * 1000))
                            .to_string(),
                    ]) + ","
                        + &instance.value_set.clone().to_string()
                        + "\n")
                        .as_bytes(),
                )
                .await?;
            }
        }
        Ok(())
    }

    async fn handle_command(&mut self, (span_id, span): (u32, Span)) {
        if let Err(e) = self.handle_command_internal(span_id, span).await {
            self.error_channel.send(e).await.unwrap();
        }
    }

    async fn handle_stop(&mut self) {
        if let Err(e) = self.fd_map.flush().await {
            self.error_channel.send(e).await.unwrap();
        }
    }

    pub async fn run(mut self) {
        loop {
            tokio::select! {
                res = self.channel.recv() => self.handle_command(res.unwrap()).await,
                _ = &mut self.kill_channel => {
                    self.handle_stop().await;
                    break
                }
            }
        }
    }
}
