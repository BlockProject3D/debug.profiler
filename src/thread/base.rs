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

use tokio::task::JoinHandle;
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::sync::oneshot;
use druid::ExtEventSink;
use async_trait::async_trait;

#[async_trait]
pub trait Run: Send {
    async fn run(&mut self);
}

pub trait Connection where Self: Sized {
    type Message: Send;
    type Worker: Run;
    type Core: Run;
    type Parameters;

    fn max_messages(&self) -> usize;
    fn new_worker(&self, channel: Sender<Self::Message>) -> Self::Worker;
    fn new_core(&self, sink: ExtEventSink, channel: Receiver<Self::Message>) -> Self::Core;
    fn new(params: Self::Parameters) -> Option<Self>;
}

pub struct BaseConnection {
    worker_task_handle: JoinHandle<()>,
    core_task_handle: JoinHandle<()>,
    worker_exit_flag: oneshot::Sender<()>,
    core_exit_flag: oneshot::Sender<()>
}

impl BaseConnection {
    pub fn new<T: 'static + Connection>(sink: ExtEventSink, params: T::Parameters) -> Option<BaseConnection> {
        let inner = T::new(params)?;
        let (worker_exit_flag, worker_exit_flag_recv) = oneshot::channel();
        let (core_exit_flag, core_exit_flag_recv) = oneshot::channel();
        let (sender, receiver) = channel(inner.max_messages());
        let mut core = inner.new_core(sink, receiver);
        let core_task_handle = tokio::spawn(async move {
            tokio::select! {
                _ = core.run() => (),
                _ = core_exit_flag_recv => ()
            }
        });
        let mut worker = inner.new_worker(sender);
        let worker_task_handle = tokio::spawn(async move {
            tokio::select! {
                _ = worker.run() => (),
                _ = worker_exit_flag_recv => ()
            }
        });
        Some(BaseConnection {
            worker_task_handle,
            core_task_handle,
            worker_exit_flag,
            core_exit_flag
        })
    }

    pub async fn end(self) {
        let _ = self.core_exit_flag.send(());
        let _ = self.worker_exit_flag.send(());
        self.worker_task_handle.await.unwrap();
        self.core_task_handle.await.unwrap();
    }
}

pub struct ConnectionWrapper(Option<BaseConnection>);

impl ConnectionWrapper {
    pub fn none() -> Self {
        Self(None)
    }

    pub fn new<T, F: FnOnce(ExtEventSink, T) -> Option<BaseConnection>>(func: F, sink: ExtEventSink, params: T) -> Self {
        Self(func(sink, params))
    }

    pub fn is_some(&self) -> bool {
        self.0.is_some()
    }

    pub async fn end(&mut self) {
        if let Some(v) = self.0.take() {
            v.end().await;
        }
    }
}
