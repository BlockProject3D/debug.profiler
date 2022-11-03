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

use serde::{Serialize, Deserialize};

#[derive(Debug, PartialEq, Clone, Copy, Eq, Serialize, Deserialize)]
pub enum Level {
    Trace,
    Debug,
    Info,
    Warning,
    Error
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Metadata {
    pub name: String, //The name of the span/event
    pub target: String, //The target of the span/event (usually this contains module path)
    pub level: Level, //The log level of the span/event
    pub module_path: Option<String>, //The module path (including crate name)
    pub file: Option<String>, //The file path
    pub line: Option<u32> //The line number in the file
}

impl Metadata {
    pub fn get_target_module(&self) -> (&str, Option<&str>) {
        let base_string = self.module_path.as_deref().unwrap_or_else(|| &*self.target);
        let target = base_string
            .find("::")
            .map(|v| &base_string[..v])
            .unwrap_or(&base_string);
        let module = base_string.find("::").map(|v| &base_string[(v + 2)..]);
        (target, module)
    }
}

impl Default for Metadata {
    fn default() -> Self {
        Metadata {
            name: "root".into(),
            target: "".into(),
            level: Level::Info,
            file: None,
            module_path: None,
            line: None
        }
    }
}
