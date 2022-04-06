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

use std::sync::Arc;
use druid::im::Vector;
use druid::Selector;
use crate::network_types::Command as NetCommand;
use crate::state::{Event, SpanLogEntry};

pub const CONNECTION_ERROR: Selector<String> = Selector::new("network.connect.error");
// The bool here is useless, it's only here to comply with druid design stupidities:
// druid REQUIRES a payload to be given even if there are NO payloads when using ExtEventSink!
pub const CONNECTION_SUCCESS: Selector<bool> = Selector::new("network.connect.success");
pub const NETWORK_ERROR: Selector<String> = Selector::new("network.error");
pub const NETWORK_COMMAND: Selector<Box<[NetCommand]>> = Selector::new("network.command");

pub const CONNECT: Selector = Selector::new("network.connect");

pub const SELECT_NODE: Selector<u64> = Selector::new("node.select");

pub const NODE_OPEN_EVENTS: Selector<Vector<Arc<Event>>> = Selector::new("node.open.events");
pub const NODE_OPEN_HISTORY: Selector<Vector<SpanLogEntry>> = Selector::new("node.open.history");
