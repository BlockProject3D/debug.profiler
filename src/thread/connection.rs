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

use std::net::TcpStream;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::JoinHandle;
use std::time::Duration;
use bincode::ErrorKind;
use crossbeam_channel::bounded;
use druid::{ExtEventSink, Target};
use crate::command::{CONNECTION_ERROR, CONNECTION_SUCCESS, NETWORK_COMMAND, NETWORK_ERROR};
use crate::thread::NET_READ_DURATION;
use super::network_types::Command as NetCommand;

type WorkerChannel = crossbeam_channel::Sender<Result<NetCommand, String>>;

struct Worker {
    stream: TcpStream,
    flag: Arc<AtomicBool>,
    channel: WorkerChannel
}

impl Worker {
    pub fn new(stream: TcpStream, channel: WorkerChannel, flag: Arc<AtomicBool>) -> Worker {
        Worker {
            stream,
            channel,
            flag
        }
    }

    pub fn connect(ip: String) -> Result<TcpStream, String> {
        let socket = TcpStream::connect(&ip).map_err(|e| e.to_string())?;
        socket.set_read_timeout(Some(NET_READ_DURATION)).map_err(|e| e.to_string())?;
        Ok(socket)
    }

    pub fn try_read_command(&mut self) -> Result<Option<NetCommand>, String> {
        match bincode::deserialize_from(&mut self.stream) {
            Ok(v) => Ok(Some(v)),
            Err(e) => {
                if let ErrorKind::Io(e) = &*e {
                    if e.kind() == std::io::ErrorKind::WouldBlock
                        || e.kind() == std::io::ErrorKind::TimedOut {
                        return Ok(None)
                    }
                }
                Err(e.to_string())
            }
        }
    }

    pub fn read_command(&mut self) -> bool {
        match self.try_read_command() {
            Ok(v) => {
                if let Some(v) = v {
                    self.channel.send(Ok(v)).unwrap();
                    return false
                }
            },
            Err(e) => {
                self.channel.send(Err(e)).unwrap();
                return true
            }
        }
        false
    }

    pub fn run(&mut self) {
        loop {
            if self.read_command() || self.flag.load(Ordering::Relaxed) {
                break
            }
        }
    }
}

pub struct Connection {
    channel: crossbeam_channel::Receiver<Result<NetCommand, String>>,
    thread_handle: JoinHandle<()>,
    exit_flag: Arc<AtomicBool>,
    sink: ExtEventSink,
    buffer: super::command::Buffer
}

impl Connection {
    pub fn new(ip: String, sink: ExtEventSink) -> Option<Self> {
        let socket = match Worker::connect(ip) {
            Ok(v) => v,
            Err(e) => {
                sink.submit_command(CONNECTION_ERROR, e, Target::Auto).unwrap();
                return None;
            }
        };
        sink.submit_command(CONNECTION_SUCCESS, false, Target::Auto).unwrap();
        let (sender, receiver) = bounded(super::command::MAX_BUFFER);
        let exit_flag = Arc::new(AtomicBool::new(false));
        let bullshitrust = exit_flag.clone();
        let thread_handle = std::thread::spawn(|| {
            let mut worker = Worker::new(socket, sender, bullshitrust);
            worker.run();
        });
        Some(Self {
            channel: receiver,
            thread_handle,
            exit_flag,
            sink,
            buffer: super::command::Buffer::new()
        })
    }

    pub fn end(self) {
        while self.channel.try_recv().is_ok() {} //Force empty the channel before attempting to join
        self.exit_flag.store(true, Ordering::Relaxed);
        self.thread_handle.join().unwrap();
    }

    pub fn step(mut self) -> Option<Self> {
        if let Err(e) = self.buffer.try_submit(&self.channel) {
            self.sink.submit_command(NETWORK_ERROR, e, Target::Auto).unwrap();
            self.end();
            return None;
        }
        if let Some(batch) = self.buffer.fast_forward() {
            self.sink.submit_command(NETWORK_COMMAND, batch, Target::Auto).unwrap();
        }
        if self.buffer.should_terminate() {
            self.end();
            return None;
        }
        return Some(self);
    }
}
