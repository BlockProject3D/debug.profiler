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

use std::collections::VecDeque;
use crossbeam_channel::Receiver;
use druid::ExtEventSink;
use crate::thread::DEFAULT_MAX_SUB_BUFFER;
use super::network_types::Command as NetCommand;

pub enum Command {
    Connect {
        ip: String,
        sink: ExtEventSink,
        max_sub_buffer: Option<usize>
    },
    StartAutoDiscovery(ExtEventSink),
    Disconnect,
    Terminate
}

pub struct Buffer {
    queue: VecDeque<NetCommand>,
    terminate_received: bool,
    max_sub_buffer: usize,
    max_buffer: usize
}

impl Buffer {
    pub fn new(max_sub_buffer: Option<usize>) -> Buffer {
        let max_sub_buffer = max_sub_buffer.unwrap_or(DEFAULT_MAX_SUB_BUFFER);
        let max_buffer = max_sub_buffer * 2;
        Buffer {
            queue: VecDeque::with_capacity(max_buffer),
            terminate_received: false,
            max_sub_buffer,
            max_buffer
        }
    }

    pub fn max_buffer(&self) -> usize {
        self.max_buffer
    }

    pub fn should_terminate(&self) -> bool {
        self.terminate_received
    }

    pub fn try_submit(&mut self, channel: &Receiver<Result<NetCommand, String>>) -> Result<(), String> {
        while self.queue.len() < self.max_buffer {
            match channel.try_recv() {
                Ok(msg) => match msg {
                    Ok(v) => {
                        if v.is_terminate() {
                            self.terminate_received = true;
                            self.queue.push_back(v);
                            break;
                        }
                        self.queue.push_back(v)
                    },
                    Err(e) => return Err(e)
                },
                Err(_) => break
            }
        }
        Ok(())
    }

    pub fn fast_forward(&mut self) -> Option<Box<[NetCommand]>> {
        if self.queue.len() > 1 {
            let mut fast_forward = Vec::with_capacity(self.max_sub_buffer);
            while fast_forward.len() < self.max_sub_buffer || self.should_terminate() {
                if let Some(net) = self.queue.pop_front() {
                    fast_forward.push(net);
                } else {
                    break;
                }
            }
            Some(fast_forward.into_boxed_slice())
        } else {
            None
        }
    }
}
