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

use std::{collections::HashMap, sync::Arc};

use crate::network_types as nt;
use crate::network_types::Duration;

use super::utils::ValueSet;

pub struct SpanInstance {
    pub message: Option<String>,
    pub value_set: ValueSet,
    pub duration: Duration,
    pub active: bool,
}

pub struct SpanData {
    pub metadata: Arc<nt::Metadata>,
    pub last_instance: Option<SpanInstance>,
    instances: HashMap<u32, SpanInstance>,
}

impl SpanData {
    pub fn is_dropped(&self) -> bool {
        self.instances.is_empty()
    }

    pub fn is_active(&self) -> bool {
        self.instances.iter().any(|(_, v)| v.active)
    }
}

pub struct SpanState {
    spans: HashMap<u32, SpanData>,
}

impl SpanState {
    pub fn new() -> Self {
        Self {
            spans: HashMap::new(),
        }
    }

    pub fn get_data(&self, id: u32) -> Option<&SpanData> {
        self.spans.get(&id)
    }

    pub fn get_any_instance(&self, id: u32) -> Option<&SpanInstance> {
        let span = self.spans.get(&id)?;
        span.instances
            .iter()
            .next()
            .map(|(_, v)| v)
            .or_else(|| span.last_instance.as_ref())
    }

    pub fn get_instance(&self, span: &nt::SpanId) -> Option<&SpanInstance> {
        self.spans.get(&span.id)?.instances.get(&span.instance)
    }

    pub fn alloc_span(&mut self, id: u32, metadata: Arc<nt::Metadata>) {
        self.spans.insert(
            id,
            SpanData {
                metadata,
                last_instance: None,
                instances: HashMap::new(),
            },
        );
    }

    pub fn alloc_instance(&mut self, span: &nt::SpanId, instance: SpanInstance) {
        if let Some(data) = self.spans.get_mut(&span.id) {
            data.instances.insert(span.instance, instance);
        }
    }

    pub fn get_instance_mut(&mut self, span: &nt::SpanId) -> Option<&mut SpanInstance> {
        self.spans
            .get_mut(&span.id)?
            .instances
            .get_mut(&span.instance)
    }

    pub fn free_instance(&mut self, span: &nt::SpanId) -> Option<SpanInstance> {
        self.spans
            .get_mut(&span.id)?
            .instances
            .remove(&span.instance)
    }
}
