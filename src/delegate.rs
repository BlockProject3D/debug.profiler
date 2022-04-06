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

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::mpsc::Sender;
use druid::{AppDelegate, Command, DelegateCtx, Env, Handled, Target, WindowDesc, WindowId};
use druid::commands::{CLOSE_WINDOW, QUIT_APP};
use druid::im::Vector;
use time::macros::format_description;
use time::OffsetDateTime;
use time_tz::OffsetDateTimeExt;
use crate::command::{CONNECT, CONNECTION_ERROR, CONNECTION_SUCCESS, NETWORK_COMMAND, NETWORK_ERROR, NODE_OPEN_EVENTS, NODE_OPEN_HISTORY, SELECT_NODE};
use crate::state::{Event, Span, SpanData, SpanLogEntry, State, StateEvents, StateHistory};
use crate::network_types::{Command as NetCommand, Level, Metadata};
use crate::view::events_window::events_window;
use crate::view::history_window::history_window;

enum WindowHandle {
    Event(usize),
    History(usize)
}

pub struct Delegate {
    channel: Sender<crate::thread::Command>,
    networked: bool,
    hack_window_handles: HashMap<WindowId, WindowHandle>
}

fn extract_target_module(record: &Metadata) -> (&str, Option<&str>) {
    let base_string = record.module_path.as_deref().unwrap_or_else(|| &*record.target);
    let target = base_string
        .find("::")
        .map(|v| &base_string[..v])
        .unwrap_or(&base_string);
    let module = base_string.find("::").map(|v| &base_string[(v + 2)..]);
    (target, module)
}

impl Delegate {
    pub fn new(channel: Sender<crate::thread::Command>) -> Delegate {
        Delegate {
            channel,
            networked: false,
            hack_window_handles: HashMap::new()
        }
    }

    fn handle_net_command(&mut self, cmd: &NetCommand, state: &mut State) {
        match cmd {
            NetCommand::SpanAlloc { id, metadata } => {
                let metadata = Arc::new(metadata.clone());
                state.tree_data.insert(*id, SpanData {
                    active: false,
                    dropped: false,
                    metadata: metadata.clone(),
                    current: SpanLogEntry::new(),
                    history: Vector::new()
                });
                state.tree.add_node(Span::with_metadata(*id, metadata));
            }
            NetCommand::SpanInit { span, parent, value_set } => {
                if let Some(parent) = parent {
                    state.tree.relocate_node(*span, *parent);
                }
                let data = state.tree_data.get_mut(span).unwrap();
                for (k, v) in value_set {
                    data.current.values.insert(k.clone(), v.clone());
                }
                data.dropped = false;
            }
            NetCommand::SpanFollows { span, follows } => {
                if let Some(parent) = state.tree.find_parent(*follows) {
                    state.tree.relocate_node(*span, parent);
                }
            }
            NetCommand::SpanValues { span, value_set } => {
                let data = state.tree_data.get_mut(span).unwrap();
                for (k, v) in value_set {
                    data.current.values.insert(k.clone(), v.clone());
                }
            }
            NetCommand::Event { span, metadata, time, value_set } => {
                let timezone = time_tz::system::get_timezone().unwrap_or(time_tz::timezones::db::america::NEW_YORK);
                let datetime = OffsetDateTime::from_unix_timestamp(*time).unwrap_or(OffsetDateTime::now_utc());
                let converted = datetime.to_timezone(timezone);
                let format = format_description!("[weekday repr:short] [month repr:short] [day] [hour repr:12]:[minute]:[second] [period case:upper]");
                let formatted = converted.format(format).unwrap_or("<error>".into());
                let level = match metadata.level {
                    Level::Trace => "TRACE",
                    Level::Debug => "DEBUG",
                    Level::Info => "INFO",
                    Level::Warning => "WARNING",
                    Level::Error => "ERROR"
                };
                let (_, module) = extract_target_module(metadata);
                let msg = format!("[{}] ({}) {}: {}", level, formatted, module.unwrap_or("main"), metadata.name);
                let event = Event {
                    msg,
                    values: value_set.clone().into_boxed_slice().into()
                };
                if let Some(span) = span {
                    let data = state.tree_data.get_mut(span).unwrap();
                    data.current.events.push_back(Arc::new(event));
                } else {
                    if !state.tree_data.contains_key(&0) {
                        state.tree_data.insert(0, SpanData::default());
                    }
                    let data = state.tree_data.get_mut(&0).unwrap();
                    data.current.events.push_back(Arc::new(event));
                }
            }
            NetCommand::SpanEnter(span) => {
                let data = state.tree_data.get_mut(span).unwrap();
                data.active = true;
            }
            NetCommand::SpanExit { span, duration } => {
                let data = state.tree_data.get_mut(span).unwrap();
                data.active = false;
                data.current.duration = *duration;
            }
            NetCommand::SpanFree(span) => {
                let data = state.tree_data.get_mut(span).unwrap();
                let log = std::mem::replace(&mut data.current, SpanLogEntry::new());
                data.dropped = true;
                data.history.push_back(log);
            }
            NetCommand::Terminate => {
                state.status = "Target application has terminated!".into();
                //Send message to kill network thread.
                self.channel.send(crate::thread::Command::Terminate).unwrap();
                //Turn off network handling.
                self.networked = false;
            }
        }
    }
}

impl AppDelegate<State> for Delegate {
    fn command(&mut self, ctx: &mut DelegateCtx, _: Target, cmd: &Command, state: &mut State, _: &Env) -> Handled {
        if cmd.is(CLOSE_WINDOW) && self.hack_window_handles.len() == 0 {
            ctx.submit_command(QUIT_APP);
            return Handled::Yes;
        } else if cmd.is(CONNECT) {
            state.status = "Connecting...".into();
            let ip = std::mem::replace(&mut state.address, String::new());
            self.channel.send(crate::thread::Command::Connect { ip, sink: ctx.get_external_handle() }).unwrap();
            return Handled::Yes;
        } else if cmd.is(CONNECTION_SUCCESS) {
            state.status = "Ready.".into();
            state.connected = true;
            self.networked = true;
            return Handled::Yes;
        }
        if let Some(err) = cmd.get(CONNECTION_ERROR) {
            state.status = format!("Connection error: {}", err);
            return Handled::Yes;
        }
        if let Some(id) = cmd.get(SELECT_NODE) {
            state.selected = *id;
            return Handled::Yes;
        }
        if let Some(events) = cmd.get(NODE_OPEN_EVENTS) {
            if let Some(event) = events.iter().nth(0) {
                let handle = state.event_windows.insert(StateEvents {
                    events: events.clone(),
                    selected_event: event.clone()
                });
                let desc = WindowDesc::new(events_window(handle));
                self.hack_window_handles.insert(desc.id, WindowHandle::Event(handle));
                ctx.new_window(desc);
            } else {
                state.status = "No events are available for this node at this time.".into()
            }
        }
        if let Some(history) = cmd.get(NODE_OPEN_HISTORY) {
            if let Some(entry) = history.iter().nth(0) {
                let handle = state.history_windows.insert(StateHistory {
                    history: history.clone(),
                    selected_history: entry.clone()
                });
                let desc = WindowDesc::new(history_window(handle));
                self.hack_window_handles.insert(desc.id, WindowHandle::History(handle));
                ctx.new_window(desc);
            } else {
                state.status = "No history is available for this node at this time.".into()
            }
        }
        if self.networked {
            if let Some(msg) = cmd.get(NETWORK_COMMAND) {
                for v in msg.iter() {
                    self.handle_net_command(v, state);
                }
                return Handled::Yes;
            }
            if let Some(err) = cmd.get(NETWORK_ERROR) {
                state.status = format!("Network error: {}", err);
                return Handled::Yes;
            }
        }
        Handled::No
    }

    fn window_removed(&mut self, id: WindowId, data: &mut State, _: &Env, _: &mut DelegateCtx) {
        if let Some(handle) = self.hack_window_handles.remove(&id) {
            match handle {
                WindowHandle::Event(h) => data.event_windows.remove(h),
                WindowHandle::History(h) => data.history_windows.remove(h)
            }
        }
    }
}
