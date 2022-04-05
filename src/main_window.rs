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

use druid::{Color, Env, EventCtx, FontDescriptor, FontFamily, FontWeight, UnitPoint, Widget, WidgetExt};
use druid::widget::{Align, Button, Flex, Label, List, Padding, TextBox, ViewSwitcher};
use druid_widget_nursery::Tree;
use crate::command::CONNECT;
use crate::network_types::Level;
use crate::state::{Span, State};
use crate::tree_widget::TreeNodeWidget;

fn handle_connect(ctx: &mut EventCtx, _: &mut State, _: &Env) {
    ctx.submit_command(CONNECT);
}

fn build_tree() -> impl Widget<Span> {
    Tree::new(|| TreeNodeWidget::new(), Span::expanded)
}

pub fn view_active() -> impl Widget<State> {
    ViewSwitcher::new(|data: &State, _| {
        data.tree_data.get(&data.selected).cloned().unwrap_or_default()
    }, |data, _, _| {
        let active = match data.active {
            true => Label::new("Yes").with_text_color(Color::rgb8(0, 255, 0)),
            false => Label::new("No").with_text_color(Color::rgb8(255, 0, 0))
        };
        let dropped = match data.dropped {
            true => Label::new("Yes").with_text_color(Color::rgb8(0, 255, 0)),
            false => Label::new("No").with_text_color(Color::rgb8(255, 0, 0))
        };
        let level = match data.metadata.level {
            Level::Trace => Label::new("Trace"),
            Level::Debug => Label::new("Debug").with_text_color(Color::rgb8(0, 255, 255)),
            Level::Info => Label::new("Info").with_text_color(Color::rgb8(0, 255, 0)),
            Level::Warning => Label::new("Debug").with_text_color(Color::rgb8(255, 255, 0)),
            Level::Error => Label::new("Debug").with_text_color(Color::rgb8(255, 0, 0))
        };
        let font = FontDescriptor::new(FontFamily::SYSTEM_UI)
            .with_weight(FontWeight::BOLD)
            .with_size(20.0);
        let mut basic = Flex::column()
            .with_child(Label::new("Basic").with_font(font.clone()))
            .with_spacer(5.0)
            .with_child(Label::new(data.metadata.name.clone()))
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
            .with_child(Label::new(format!("Duration: {}", data.current.duration)));
        if let Some(file) = &data.metadata.file {
            basic.add_child(Label::new(format!("File: {} [{:?}]", file, data.metadata.line)))
        }
        if let Some(module_path) = &data.metadata.module_path {
            basic.add_child(Label::new(format!("Module path: {}", module_path)))
        }
        let mut values = Flex::column()
            .with_child(Label::new("Values").with_font(font.clone()))
            .with_spacer(5.0);
        for (name, value) in &data.current.values {
            values.add_child(Label::new(format!("{}: {}", name, value)))
        }
        Box::new(
            Flex::column()
                .with_child(basic.border(Color::BLACK, 0.5))
                .with_spacer(10.0)
                .with_child(values.border(Color::BLACK, 0.5))
        )
    })
}

fn build_view() -> impl Widget<State> {
    Flex::column()
        .with_child(Label::dynamic(|data: &State, _| format!("Selected node: {}", data.selected)))
        .with_child(Padding::new(20.0, view_active()))
        .expand_width()
        .border(Color::BLACK, 0.5)
}

pub fn ui_builder() -> impl Widget<State> {
    ViewSwitcher::new(|data: &State, _| data.connected, |connected, _, _| {
        let flex = match connected {
            true => Flex::column()
                .with_child(Flex::row()
                    .with_child(build_tree().lens(State::tree).border(Color::BLACK, 0.5))
                    .with_spacer(5.0)
                    .with_flex_child(build_view(), 50.0)
                    .expand_width()),
            false => Flex::column()
                .with_child(Label::new("Please enter the ip address of the application to debug:"))
                .with_spacer(15.0)
                .with_child(TextBox::new().lens(State::address))
                .with_spacer(5.0)
                .with_child(Button::new("Connect").on_click(handle_connect).padding(5.0))
        };
        Box::new(Padding::new(10.0, Flex::column()
            .with_flex_child(Align::centered(flex), 90.0)
            .with_flex_child(Align::vertical(UnitPoint::BOTTOM, Label::dynamic(|data: &State, _| data.status.clone())), 5.0)))
    })
}
