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

use std::net::IpAddr;
use druid::{Data, Lens};
use druid::im::{HashMap, Vector};
use crate::state::{Preferences, Span, SpanData, StateEvents, StateHistory};
use crate::state::window_map::WindowMap;

#[derive(Debug, Clone, Data)]
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
