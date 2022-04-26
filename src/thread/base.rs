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

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;
//use crossbeam_channel::{bounded, Receiver, Sender};
use tokio::sync::mpsc::{channel, Receiver, Sender};
use tokio::sync::oneshot;
use druid::ExtEventSink;
use async_trait::async_trait;

/*pub struct BaseWorker<T: Connection> {
    channel: Sender<T::Message>,
    exit_flag: Arc<AtomicBool>
}

impl<T: Connection> BaseWorker<T> {
    fn new(channel: Sender<T::Message>, exit_flag: Arc<AtomicBool>) -> BaseWorker<T> {
        BaseWorker {
            channel,
            exit_flag
        }
    }

    pub fn should_exit(&self) -> bool {
        self.exit_flag.load(Ordering::Relaxed)
    }

    pub fn send(&self, msg: T::Message) {
        self.channel.send(msg).unwrap(); //FIXME: For some weird reasons this randomly panics.
    }
}*/

#[async_trait]
pub trait Run<T: Connection>: Send {
    async fn run(&mut self);
}

pub trait Connection where Self: Sized {
    type Message: Send;
    type Worker: Run<Self>;
    type Parameters;

    fn max_messages(&self) -> usize;
    fn new_worker(&self, channel: Sender<Self::Message>) -> Self::Worker;
    fn new(params: Self::Parameters) -> Option<Self>;
    fn step(&mut self, sink: &ExtEventSink, channel: &mut Receiver<Self::Message>) -> bool;
}

pub struct BaseConnection<T: Connection> {
    channel: Receiver<T::Message>,
    thread_handle: JoinHandle<()>,
    //exit_flag: Arc<AtomicBool>,
    exit_flag: Option<oneshot::Sender<()>>,
    sink: ExtEventSink,
    inner: T
}

impl<T: 'static + Connection> BaseConnection<T> {
    pub fn new(sink: ExtEventSink, params: T::Parameters) -> Option<BaseConnection<T>> {
        let inner = T::new(params)?;
        //let exit_flag = Arc::new(AtomicBool::new(false));
        let (exit_flag, exit_flag1) = oneshot::channel();
        let (sender, receiver) = channel(inner.max_messages());
        let mut worker = inner.new_worker(sender);
        //let base = BaseWorker::new(sender, exit_flag.clone());
        let thread_handle = std::thread::spawn(move || {
            let runtime = tokio::runtime::Builder::new_current_thread().enable_io().build().unwrap();
            runtime.block_on(async {
                tokio::select! {
                    _ = worker.run() => (),
                    _ = exit_flag1 => ()
                }
            });
        });
        Some(BaseConnection {
            sink,
            channel: receiver,
            exit_flag: Some(exit_flag),
            thread_handle,
            inner
        })
    }

    pub fn end(mut self) {
        while self.channel.try_recv().is_ok() {} //Force empty the channel before attempting to join
        self.exit_flag.take().map(|v| v.send(()));
        //self.exit_flag.store(true, Ordering::Relaxed);
        self.thread_handle.join().unwrap();
    }

    pub fn step(mut self) -> Option<Self> {
        if !self.inner.step(&self.sink, &mut self.channel) {
            self.end();
            return None;
        }
        Some(self)
    }
}

macro_rules! hack_rust_garbage_private_rule {
    ($name: ident<$params: ty, $inner: ty>) => {
        pub struct $name(crate::thread::base::BaseConnection<$inner>);
        impl $name {
            pub fn new(sink: ExtEventSink, params: $params) -> Option<$name> {
                crate::thread::base::BaseConnection::<$inner>::new(sink, params).map($name)
            }
            pub fn end(self) {
                self.0.end()
            }
            pub fn step(self) -> Option<Self> {
                self.0.step().map($name)
            }
        }
    };
}
pub(crate) use hack_rust_garbage_private_rule;
