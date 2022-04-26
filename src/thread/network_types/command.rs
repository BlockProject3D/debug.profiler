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
use super::Metadata;
use super::Value;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpanId {
    pub id: u32,
    pub instance: u32
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum Command {
    SpanAlloc {
        id: SpanId,
        metadata: Metadata
    },

    SpanInit {
        span: SpanId,
        parent: Option<SpanId>, //None must mean that span is at root
        message: Option<String>,
        value_set: Vec<(String, Value)>
    },

    SpanFollows {
        span: SpanId,
        follows: SpanId
    },

    SpanValues {
        span: SpanId,
        message: Option<String>,
        value_set: Vec<(String, Value)>
    },

    Event {
        span: Option<SpanId>,
        metadata: Metadata,
        time: i64,
        message: Option<String>,
        value_set: Vec<(String, Value)>
    },

    SpanEnter(SpanId),

    SpanExit {
        span: SpanId,
        duration: f64
    },

    SpanFree(SpanId),

    Terminate
}

impl Command {
    pub fn is_terminate(&self) -> bool { // The target application has requested termination.
        match self {
            Command::Terminate => true,
            _ => false
        }
    }
}