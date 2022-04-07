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

use druid::{Color, Widget, WidgetExt};
use druid::widget::{Button, Flex, Label, Padding, ViewSwitcher};
use crate::command::SPAWN_WINDOW;
use crate::network_types::Level;
use crate::state::State;
use crate::window::EventsWindow;
use crate::window::HistoryWindow;

pub fn view_active() -> impl Widget<State> {
    let active = ViewSwitcher::new(
        |data: &State, _| data.get_field(|v| v.active),
        |active, _, _| Box::new(super::common::build_bool_view(*active)));
    let dropped = ViewSwitcher::new(
        |data: &State, _| data.get_field(|v| v.dropped),
        |dropped, _, _| Box::new(super::common::build_bool_view(*dropped)));
    let level = ViewSwitcher::new(
        |data: &State, _| data.get_field(|v| v.metadata.clone()),
        |metadata, _, _| {
            Box::new(match metadata.level {
                Level::Trace => Label::new("Trace"),
                Level::Debug => Label::new("Debug").with_text_color(Color::rgb8(0, 255, 255)),
                Level::Info => Label::new("Info").with_text_color(Color::rgb8(0, 255, 0)),
                Level::Warning => Label::new("Debug").with_text_color(Color::rgb8(255, 255, 0)),
                Level::Error => Label::new("Debug").with_text_color(Color::rgb8(255, 0, 0))
            })
    });
    let name = ViewSwitcher::new(
        |data: &State, _| data.get_field(|v| v.metadata.clone()),
        |metadata, _, _| Box::new(Label::new(metadata.name.clone())));
    let file_module_path = ViewSwitcher::new(
        |data: &State, _| data.get_field(|v| v.metadata.clone()),
        |metadata, _, _| {
            let mut flex = Flex::column();
            if let Some(file) = &metadata.file {
                flex.add_child(Label::new(format!("File: {} [{:?}]", file, metadata.line)));
            }
            if let Some(module_path) = &metadata.module_path {
                flex.add_child(Label::new(format!("Module path: {}", module_path)));
            }
            Box::new(flex)
        });
    let duration = ViewSwitcher::new(
        |data: &State, _| data.get_field(|v| v.current.duration),
        |duration, _, _| Box::new(Label::new(format!("Duration: {}s", duration))));
    let values = ViewSwitcher::new(
        |data: &State, _| data.get_field(|v| v.current.values.clone()),
        |values, _, _| Box::new(super::common::build_values_view(values.iter())));

    let basic = super::common::build_box("Basic")
        .with_child(name)
        .with_child(
            Flex::row()
                .with_child(Label::new("Active: "))
                .with_child(active)
        )
        .with_child(
            Flex::row()
                .with_child(Label::new("Dropped: "))
                .with_child(dropped)
        )
        .with_child(
            Flex::row()
                .with_child(Label::new("Level: "))
                .with_child(level)
        )
        .with_child(duration)
        .with_child(file_module_path);
    let values = super::common::build_box("Values").with_child(values);
    let actions = Flex::row()
        .with_child(
            Button::new("View events")
                .on_click(|ctx, data: &mut State, _| {
                    let events = data.tree_data.get(&data.selected).unwrap().current.events.clone();
                    if let Some(window) = EventsWindow::new(data, events) {
                        ctx.submit_command(SPAWN_WINDOW.with(Box::new(window)));
                    }
                })
        )
        .with_spacer(5.0)
        .with_child(
            Button::new("View history")
                .on_click(|ctx, data: &mut State, _| {
                    let history = data.tree_data.get(&data.selected).unwrap().history.clone();
                    if let Some(window) = HistoryWindow::new(data, history) {
                        ctx.submit_command(SPAWN_WINDOW.with(Box::new(window)));
                    }
                })
        );
    Flex::column()
        .with_child(basic.border(Color::BLACK, 0.5))
        .with_spacer(10.0)
        .with_child(values.border(Color::BLACK, 0.5))
        .with_spacer(10.0)
        .with_child(Padding::new(5.0, actions).border(Color::BLACK, 0.5))
}
