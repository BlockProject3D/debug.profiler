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

use std::borrow::Cow;
use std::sync::Arc;
use druid::Data;
use druid::im::{HashMap, Vector};
use crate::thread::network_types::{Metadata, Value};

#[derive(Clone, Data)]
pub struct Event {
    pub msg: String,
    pub values: Arc<[(String, Value)]>
}

#[derive(Default, Clone, Data)]
pub struct SpanLogEntry {
    pub duration: f64, //The last duration in seconds of this span
    pub values: HashMap<String, Value>, //All values that have been set as part of this span
    pub events: Vector<Arc<Event>>, //All events that are in this span
}

impl SpanLogEntry {
    pub fn new() -> SpanLogEntry {
        SpanLogEntry::default()
    }
}

#[derive(Default, Clone, Data)]
pub struct SpanData {
    pub active: bool, //Is this span currently entered
    pub dropped: bool,
    pub metadata: Arc<Metadata>,
    instances: HashMap<u32, SpanLogEntry>,
    last_instance: u32,
    pub history: Vector<SpanLogEntry> //The history of previously dropped instances of the span
}

impl SpanData {
    pub fn new(metadata: Arc<Metadata>) -> SpanData {
        SpanData {
            active: false,
            dropped: false,
            metadata,
            instances: HashMap::new(),
            last_instance: 0,
            history: Vector::new()
        }
    }

    pub fn instance_count(&self) -> usize {
        self.instances.len()
    }

    pub fn root_instance(&mut self) -> &mut SpanLogEntry {
        if self.instances.contains_key(&0) {
            self.instance_mut(0)
        } else {
            self.new_instance(0)
        }
    }

    pub fn new_instance(&mut self, instance: u32) -> &mut SpanLogEntry {
        self.last_instance = instance;
        self.instances.insert(instance, SpanLogEntry::new());
        return unsafe { self.instances.get_mut(&instance).unwrap_unchecked() }
    }

    pub fn current(&self) -> Cow<SpanLogEntry> {
        self.instances.get(&self.last_instance).map(Cow::Borrowed).unwrap_or_default()
    }

    pub fn instance(&self, instance: u32) -> &SpanLogEntry {
        self.instances.get(&instance).unwrap()
    }

    pub fn instance_mut(&mut self, instance: u32) -> &mut SpanLogEntry {
        self.instances.get_mut(&instance).unwrap()
    }

    pub fn free_instance(&mut self, instance: u32) -> SpanLogEntry {
        self.instances.remove(&instance).unwrap()
    }
}
