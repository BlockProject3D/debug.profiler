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

mod paths;
mod fd_map;

use std::collections::HashMap;
use std::path::{PathBuf, Path};
use std::io::Result;

use crate::network_types as nt;

use self::fd_map::FdMap;
use self::paths::Paths;

struct SpanData {
    message: Option<String>,
    value_set: Vec<(String, nt::Value)>,
    duration: f64
}

pub struct Session {
    paths: Paths,
    fd_map: FdMap,
    spans: HashMap<nt::SpanId, SpanData>
}

impl Session {
    pub async fn new(client_index: usize, max_fd_count: usize) -> Result<Session> {
        let paths = Paths::new(client_index).await?;
        Ok(Session {
            paths,
            fd_map: FdMap::new(max_fd_count),
            spans: HashMap::new()
        })
    }

    pub async fn handle_command(&mut self, cmd: nt::Command) {
        match cmd {
            nt::Command::SpanAlloc { id, metadata } => todo!(),
            nt::Command::SpanInit { span, parent, message, value_set } => todo!(),
            nt::Command::SpanFollows { span, follows } => todo!(),
            nt::Command::SpanValues { span, message, value_set } => todo!(),
            nt::Command::Event { span, metadata, time, message, value_set } => todo!(),
            nt::Command::SpanEnter(id) => todo!(),
            nt::Command::SpanExit { span, duration } => todo!(),
            nt::Command::SpanFree(id) => todo!(),
            nt::Command::Terminate => todo!(),
        }
    }
}
