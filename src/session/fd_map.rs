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
use std::path::PathBuf;

use tokio::{
    fs::{File, OpenOptions},
    io::{AsyncWriteExt, BufReader},
};

use super::paths::{Directory, Paths};

struct FdEntry {
    file: BufReader<File>,
    dir: Directory,
    span: u32,
}

pub struct FdMap {
    fd_map: Vec<FdEntry>,
    max_fd_count: usize,
}

impl FdMap {
    pub fn new(max_fd_count: usize) -> FdMap {
        FdMap {
            fd_map: Vec::with_capacity(max_fd_count),
            max_fd_count,
        }
    }

    pub async fn open_file(
        &mut self,
        paths: &Paths,
        span: u32,
        dir: Directory,
    ) -> Result<&mut BufReader<File>> {
        if let Some(entry) = self
            .fd_map
            .iter()
            .position(|v| v.span == span && v.dir == dir)
        {
            return Ok(&mut self.fd_map[entry].file);
        }
        if self.fd_map.len() >= self.max_fd_count {
            self.fd_map[0].file.flush().await?;
            self.fd_map.remove(0);
        }
        let path = paths.get(dir).join(format!("{}.csv", span));
        let file = BufReader::new(
            OpenOptions::new()
                .append(true)
                .create(true)
                .open(path)
                .await?,
        );
        self.fd_map.push(FdEntry { dir, span, file });
        Ok(self.fd_map.last_mut().map(|v| &mut v.file).unwrap())
    }
}
