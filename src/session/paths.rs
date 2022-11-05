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
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Directory {
    Runs,
    Events,
    Metadata,
}

pub struct Paths {
    dir: PathBuf,
    runs_dir: PathBuf,
    events_dir: PathBuf,
    metadata_dir: PathBuf,
}

async fn get_data_dir(client_index: usize) -> Result<PathBuf> {
    let data_dir = Path::new("./data");
    tokio::fs::create_dir(data_dir).await?;
    let client_dir = data_dir.join(format!("{}", client_index));
    tokio::fs::create_dir(&client_dir).await?;
    Ok(client_dir)
}

impl Paths {
    pub async fn new(client_index: usize) -> Result<Paths> {
        let dir = get_data_dir(client_index).await?;
        let runs_dir = dir.join("runs");
        let events_dir = dir.join("events");
        let metadata_dir = dir.join("events");
        tokio::fs::create_dir(&runs_dir).await?;
        tokio::fs::create_dir(&events_dir).await?;
        tokio::fs::create_dir(&metadata_dir).await?;
        Ok(Paths {
            dir,
            events_dir,
            runs_dir,
            metadata_dir,
        })
    }

    pub fn get(&self, dir: Directory) -> &Path {
        match dir {
            Directory::Runs => &self.runs_dir,
            Directory::Events => &self.events_dir,
            Directory::Metadata => &self.metadata_dir,
        }
    }
}
