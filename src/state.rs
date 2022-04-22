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
use std::net::IpAddr;
use std::sync::Arc;
use druid::{Data, Lens};
use druid::im::{HashMap, Vector};
use druid_widget_nursery::TreeNode;
use crate::thread::DEFAULT_MAX_SUB_BUFFER;
use crate::thread::network_types::{Metadata, Value};
use crate::window_map::WindowMap;

#[derive(Clone, Data, Debug, Lens)]
pub struct Span {
    pub metadata: Arc<Metadata>,
    pub id: u32,
    pub expanded: bool,
    children: Vector<Span>
}

impl Span {
    pub fn new() -> Span {
        Span {
            metadata: Arc::new(Metadata::default()),
            id: 0,
            expanded: true,
            children: Vector::new()
        }
    }

    pub fn with_metadata(id: u32, metadata: Arc<Metadata>) -> Span {
        Span {
            metadata,
            id,
            expanded: true,
            children: Vector::new()
        }
    }

    /// Attempts to find the parent of the specified node.
    pub fn find_parent(&self, id: u32) -> Option<u32> {
        for v in &self.children {
            if v.id == id {
                return Some(self.id);
            }
            if let Some(id) = v.find_parent(id) {
                return Some(id);
            }
        }
        None
    }

    /// Attempts to remove the specified node.
    ///
    /// If the node wasn't found, None is returned.
    /// If the node was found and removed, the removed node is returned.
    pub fn remove_node(&mut self, id: u32) -> Option<Span> {
        let index = self.children.iter().enumerate()
            .find_map(|(i, v)| if v.id == id { Some(i) } else { None });
        if let Some(index) = index {
            return Some(self.children.remove(index));
        }
        //Stupid bullshit design of im crate: not able to implement a fucking IntoIterator for &mut!
        for v in self.children.iter_mut() {
            if let Some(node) = v.remove_node(id) {
                return Some(node);
            }
        }
        None
    }

    /// Inserts a new child node to this node.
    pub fn add_node(&mut self, node: Span) {
        self.children.push_back(node);
    }

    /// Attempts to add the specified node under the specified parent.
    ///
    /// If the parent could not be found the node is returned.
    /// If the parent was found and the node added None is returned.
    pub fn add_node_with_parent(&mut self, node: Span, parent: u32) -> Option<Span> {
        if self.id == parent {
            self.add_node(node);
            return None;
        }
        let mut node = node;
        for v in self.children.iter_mut() {
            match v.add_node_with_parent(node, parent) {
                Some(v) => node = v,
                None => return None
            }
        }
        Some(node)
    }

    /// Attempts to relocated the specified node under the new specified parent.
    ///
    /// Returns true if the operation has succeeded.
    pub fn relocate_node(&mut self, id: u32, new_parent: u32) -> bool {
        if let Some(node) = self.remove_node(id) {
            if self.add_node_with_parent(node, new_parent).is_none() {
                return true;
            }
        }
        return false;
    }
}

impl Default for Span {
    fn default() -> Self {
        Self::new()
    }
}

impl TreeNode for Span {
    fn children_count(&self) -> usize {
        self.children.len()
    }

    fn get_child(&self, index: usize) -> &Self {
        &self.children[index]
    }

    fn for_child_mut(&mut self, index: usize, mut func: impl FnMut(&mut Self, usize)) {
        let data = &mut self.children[index];
        func(data, index);
    }
}

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

//The state for an events window.
#[derive(Clone, Data, Lens)]
pub struct StateEvents {
    pub selected_event: Arc<Event>,
    pub events: Vector<Arc<Event>>
}

//The state for a history window.
#[derive(Clone, Data, Lens)]
pub struct StateHistory {
    pub selected_history: SpanLogEntry,
    pub history: Vector<SpanLogEntry>
}

#[derive(Copy, Clone, Eq, PartialEq, Data)]
pub enum Theme {
    Light,
    Dark
}

impl Default for Theme {
    fn default() -> Self {
        Self::Light
    }
}

#[derive(Clone, Data, Lens)]
pub struct Preferences {
    pub max_history: u32,
    pub max_events: u32,
    pub theme: Theme,
    pub inherit: bool,
    pub max_sub_buffer: usize
}

impl Default for Preferences {
    fn default() -> Self {
        Preferences {
            max_history: 10,
            max_events: 0,
            theme: Theme::default(),
            inherit: true,
            max_sub_buffer: DEFAULT_MAX_SUB_BUFFER
        }
    }
}

#[derive(Clone, Data)]
pub struct Peer {
    pub name: String,
    pub addr: IpAddr
}

#[derive(Clone, Data, Lens, Default)]
pub struct State {
    pub tree: Span,
    pub tree_data: HashMap<u32, SpanData>,
    pub connected: bool,
    pub address: String,
    pub status: String,
    pub selected: u32,
    pub event_windows: WindowMap<StateEvents>,
    pub history_windows: WindowMap<StateHistory>,
    pub preferences: Preferences,
    pub discovered_peers: Vector<Peer>,
    pub selected_peer: Option<IpAddr>
}

impl State {
    pub fn get_field<T: Default, F: FnOnce(&SpanData) -> T>(&self, f: F) -> T {
        self.tree_data.get(&self.selected).map(f).unwrap_or_default()
    }

    pub fn reset(&mut self) {
        self.selected = 0;
        self.address = "".into();
        self.tree = Span::default();
        self.tree_data = druid::im::HashMap::new();
        self.connected = false;
        self.status = "".into();
        self.selected_peer = None;
    }
}
