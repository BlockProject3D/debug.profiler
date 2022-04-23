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

use std::fs::File;
use std::io::{BufReader, BufWriter};
use bp3d_fs::dirs::App;
use bpx::core::builder::SectionHeaderBuilder;
use bpx::core::Container;
use bpx::sd::serde::EnumSize;
use serde::{Serialize, Deserialize};
use druid::{Data, Lens};
use crate::constants::{DEFAULT_MAX_SUB_BUFFER, FILESYS_APP_NAME};

#[derive(Copy, Clone, Eq, PartialEq, Data, Serialize, Deserialize)]
pub enum Theme {
    Light,
    Dark
}

impl Default for Theme {
    fn default() -> Self {
        Self::Light
    }
}

#[derive(Clone, Data, Lens, Serialize, Deserialize)]
pub struct Preferences {
    pub max_history: u32,
    pub max_events: u32,
    pub theme: Theme,
    pub inherit: bool,
    pub max_sub_buffer: usize
}

impl Preferences {
    pub fn save(&self) {
        let bpx_header = bpx::core::builder::MainHeaderBuilder::new().ty(b'U').build();
        let container = App::new(FILESYS_APP_NAME).get_config()
            .map(|v| v.join("preferences.bpx"))
            .map_err(|_| ())
            .and_then(|v| File::create(v).map_err(|_| ()))
            .map(BufWriter::new)
            .map(|v| Container::create(v, bpx_header));
        match container {
            Ok(mut container) => {
                let hdl = container.sections_mut()
                    .create(SectionHeaderBuilder::new().ty(0x1));
                match self.serialize(bpx::sd::serde::Serializer::new(EnumSize::U8, false)) {
                    Ok(v) => {
                        let mut section = container.sections().open(hdl).unwrap();
                        let _ = v.write(&mut *section);
                    },
                    _ => ()
                }
                let _ = container.save();
            }
            _ => ()
        }
    }

    fn load() -> Option<Preferences> {
        let container = App::new(FILESYS_APP_NAME).get_config()
            .map(|v| v.join("preferences.bpx"))
            .map_err(|_| ())
            .and_then(|v| File::open(v).map_err(|_| ()))
            .map(BufReader::new)
            .and_then(|v| Container::open(v).map_err(|_| ()))
            .ok()?;
        let hdl = container.sections().find_by_type(0x1)?;
        let mut section = container.sections().load(hdl).ok()?;
        Preferences::deserialize(bpx::sd::serde::Deserializer::new(EnumSize::U8, bpx::sd::Value::read(&mut *section).ok()?)).ok()
    }
}

impl Default for Preferences {
    fn default() -> Self {
        Self::load().unwrap_or(Preferences {
            max_history: 16,
            max_events: 0,
            theme: Theme::default(),
            inherit: true,
            max_sub_buffer: DEFAULT_MAX_SUB_BUFFER
        })
    }
}
