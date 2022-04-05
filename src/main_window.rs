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

use druid::{Env, EventCtx, Widget, WidgetExt};
use druid::widget::{Button, CrossAxisAlignment, Flex, Label, MainAxisAlignment, TextBox, ViewSwitcher};
use druid_widget_nursery::Tree;
use crate::command::CONNECT;
use crate::state::{Span, State};

fn handle_connect(ctx: &mut EventCtx, _: &mut State, _: &Env) {
    ctx.submit_command(CONNECT);
}

fn build_tree() -> impl Widget<Span> {
    Tree::default(Span::expanded)
}

pub fn ui_builder() -> impl Widget<State> {
    ViewSwitcher::new(|data: &State, _| data.connected, |connected, _, _| {
        let mut flex = match connected {
            true => Flex::column()
                .with_child(Flex::row()
                    .with_child(build_tree().lens(State::tree))
                )
                .with_child(Label::new("Connected!")),
            false => Flex::column()
                .with_child(Label::new("Please enter the ip address of the application to debug:"))
                .with_spacer(15.0)
                .with_child(TextBox::new().lens(State::address))
                .with_spacer(5.0)
                .with_child(Button::new("Connect").on_click(handle_connect).padding(5.0))
        };
        flex.add_spacer(20.0);
        flex.add_child(Label::dynamic(|data: &State, _| data.status.clone()));
        Box::new(Flex::column()
            .main_axis_alignment(MainAxisAlignment::Center)
            .cross_axis_alignment(CrossAxisAlignment::Fill)
            .with_flex_child(flex, 90.0))
    })
}
