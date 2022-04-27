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
use druid::{AppDelegate, Command, DelegateCtx, Env, Handled, Target, WindowId};
use druid::commands::{CLOSE_WINDOW, QUIT_APP};
use time::macros::format_description;
use time::OffsetDateTime;
use time_tz::OffsetDateTimeExt;
use tokio::sync::mpsc::UnboundedSender;
use crate::command::{CONNECT, CONNECTION_ERROR, CONNECTION_SUCCESS, DISCONNECT, DISCOVER_START, NETWORK_COMMAND, NETWORK_ERROR, NETWORK_PEER, NETWORK_PEER_ERR, NEW, SELECT_NODE, SPAWN_WINDOW};
use crate::state::{Event, Span, SpanData, State};
use crate::thread::network_types::{Command as NetCommand, Level, Value};
use crate::window::Destroy;

pub struct Delegate {
    channel: UnboundedSender<crate::thread::Command>,
    networked: bool,
    windows: HashMap<WindowId, Box<dyn Destroy>>,
    window_count: usize
}

impl Delegate {
    pub fn new(channel: UnboundedSender<crate::thread::Command>) -> Delegate {
        Delegate {
            channel,
            networked: false,
            windows: HashMap::new(),
            window_count: 1
        }
    }

    fn handle_net_command(&mut self, cmd: &NetCommand, state: &mut State) {
        match cmd {
            NetCommand::SpanAlloc { id, metadata } => {
                let metadata = Arc::new(metadata.clone());
                state.tree_data.insert(id.id, SpanData::new(metadata.clone()));
                state.tree.add_node(Span::with_metadata(id.id, metadata));
            }
            NetCommand::SpanInit { span, parent, message, value_set } => {
                if let Some(parent) = parent {
                    state.tree.relocate_node(span.id, parent.id);
                }
                let data = state.tree_data.get_mut(&span.id).unwrap();
                let instance = data.new_instance(span.instance);
                if let Some(message) = message {
                    instance.values.insert("message".into(), Value::String(message.clone()));
                }
                for (k, v) in value_set {
                    instance.values.insert(k.clone(), v.clone());
                }
                data.dropped = false;
            }
            NetCommand::SpanFollows { span, follows } => {
                if let Some(parent) = state.tree.find_parent(follows.id) {
                    state.tree.relocate_node(span.id, parent);
                }
            }
            NetCommand::SpanValues { span, message, value_set } => {
                let data = state.tree_data.get_mut(&span.id).unwrap();
                if let Some(message) = message {
                    data.instance_mut(span.instance).values.insert("message".into(), Value::String(message.clone()));
                }
                for (k, v) in value_set {
                    data.instance_mut(span.instance).values.insert(k.clone(), v.clone());
                }
            }
            NetCommand::Event { span, metadata, time, message, value_set } => {
                let timezone = time_tz::system::get_timezone().unwrap_or(time_tz::timezones::db::us::CENTRAL);
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
                let (target, module) = metadata.get_target_module();
                let msg = format!("<{}> [{}] ({}) {}: {}", target, level, formatted, module.unwrap_or("main"), message.as_ref().unwrap_or(&metadata.name));
                let mut value_set = value_set.clone();
                let events = if let Some(span) = span {
                    let data = state.tree_data.get_mut(&span.id).unwrap();
                    if state.preferences.inherit {
                        // Potential problem if parent is freed before child. I don't expect tracing
                        // to call try_close on the parent before the child.
                        //TODO: Check if by any chance tracing would act weird...
                        let iter = data.instance(span.instance).values.iter()
                            .map(|(k, v)| (data.metadata.name.clone() + "::" + k, v.clone()));
                        for (k, v) in iter {
                            value_set.insert(0, (k, v));
                        }
                    }
                    &mut data.instance_mut(span.instance).events
                } else {
                    if !state.tree_data.contains_key(&0) {
                        state.tree_data.insert(0, SpanData::default());
                    }
                    &mut state.tree_data.get_mut(&0).unwrap().root_instance().events
                };
                let event = Event {
                    msg,
                    values: value_set.into_boxed_slice().into()
                };
                events.push_front(Arc::new(event));
                while state.preferences.max_events > 0
                    && events.len() > state.preferences.max_events as usize {
                    events.pop_back(); // Drop oldest item to ensure we do not exceed
                    // the user defined size limit.
                }
            }
            NetCommand::SpanEnter(span) => {
                let data = state.tree_data.get_mut(&span.id).unwrap();
                data.active = true;
            }
            NetCommand::SpanExit { span, duration } => {
                let data = state.tree_data.get_mut(&span.id).unwrap();
                if data.instance_count() == 0 {
                    data.active = false;
                }
                data.instance_mut(span.instance).duration = *duration;
            }
            NetCommand::SpanFree(span) => {
                let mut log = state.tree_data.get_mut(&span.id).unwrap().free_instance(span.instance);
                if state.preferences.inherit {
                    // Potential problem if parent is freed before child. I don't expect tracing
                    // to call try_close on the parent before the child.
                    //TODO: Check if by any chance tracing would act weird...
                    if let Some(parent) = state.tree.find_parent(span.id) {
                        let data1 = state.tree_data.get(&parent).unwrap();
                        let fuckingrust = data1.current();
                        let iter = fuckingrust.values.iter()
                            .map(|(k, v)| (data1.metadata.name.clone() + "::" + k, v.clone()));
                        for (k, v) in iter {
                            log.values.insert(k, v);
                        }
                    }
                }
                let data = state.tree_data.get_mut(&span.id).unwrap();
                data.dropped = true;
                data.history.push_front(log);
                while state.preferences.max_history > 0
                    && data.history.len() > state.preferences.max_history as usize {
                    data.history.pop_back(); // Drop oldest item to ensure we do not exceed the
                    // user defined size limit.
                }
            }
            NetCommand::Terminate => {
                state.status = "Target application has terminated!".into();
                //Turn off network handling.
                self.networked = false;
            }
        }
    }

    fn close_window(&mut self) -> bool {
        self.window_count -= 1;
        self.window_count == 0
    }
}

impl AppDelegate<State> for Delegate {
    fn command(&mut self, ctx: &mut DelegateCtx, _: Target, cmd: &Command, state: &mut State, _: &Env) -> Handled {
        if cmd.is(CLOSE_WINDOW) && self.close_window() {
            ctx.submit_command(QUIT_APP);
            return Handled::Yes;
        } else if cmd.is(NEW) {
            self.channel.send(crate::thread::Command::Disconnect).ok().unwrap();
            //Turn off network handling.
            self.networked = false;
            state.reset();
            return Handled::Yes;
        }
        if let Some(id) = cmd.get(SELECT_NODE) {
            state.selected = *id;
            return Handled::Yes;
        }
        if let Some(window) = cmd.get(SPAWN_WINDOW) {
            self.window_count += 1;
            let desc = window.build();
            if let Some(destructor) = window.destructor() {
                self.windows.insert(desc.id, destructor);
            }
            ctx.new_window(desc);
            return Handled::Yes;
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
                //Turn off network handling.
                self.networked = false;
                return Handled::Yes;
            }
            if cmd.is(DISCONNECT) {
                state.status = "Disconnected from target application!".into();
                self.channel.send(crate::thread::Command::Disconnect).ok().unwrap();
                //Turn off network handling.
                self.networked = false;
                return Handled::Yes;
            }
        } else {
            if let Some(peer) = cmd.get(NETWORK_PEER) {
                state.discovered_peers.push_back(peer.clone());
                return Handled::Yes;
            }
            if let Some(err) = cmd.get(NETWORK_PEER_ERR) {
                state.status = format!("Auto discovery service error: {}", err);
                return Handled::Yes;
            }
            if let Some(err) = cmd.get(CONNECTION_ERROR) {
                state.status = format!("Connection error: {}", err);
                return Handled::Yes;
            }
            if cmd.is(DISCOVER_START) {
                self.channel.send(crate::thread::Command::StartAutoDiscovery(ctx.get_external_handle())).ok().unwrap();
                return Handled::Yes;
            } else if cmd.is(CONNECT) {
                state.status = "Connecting...".into();
                let ip = std::mem::replace(&mut state.address, String::new());
                self.channel.send(crate::thread::Command::Connect {
                    ip,
                    sink: ctx.get_external_handle(),
                    max_sub_buffer: Some(state.preferences.max_sub_buffer)
                }).ok().unwrap();
                return Handled::Yes;
            } else if cmd.is(CONNECTION_SUCCESS) {
                state.status = "Ready.".into();
                state.connected = true;
                state.discovered_peers.clear();
                self.networked = true;
                return Handled::Yes;
            }
        }
        Handled::No
    }

    fn window_removed(&mut self, id: WindowId, data: &mut State, _: &Env, _: &mut DelegateCtx) {
        if let Some(destructor) = self.windows.remove(&id) {
            destructor.destroy(data);
        }
    }
}
