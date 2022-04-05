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

use druid::{Color, Env, EventCtx, UnitPoint, Widget, WidgetExt};
use druid::widget::{Align, Button, Flex, Label, Padding, TextBox, ViewSwitcher};
use crate::command::CONNECT;
use crate::state::State;
use crate::view::main::view_main;
use crate::view::tree::view_tree;

fn handle_connect(ctx: &mut EventCtx, _: &mut State, _: &Env) {
    ctx.submit_command(CONNECT);
}

pub fn main_window() -> impl Widget<State> {
    ViewSwitcher::new(|data: &State, _| data.connected, |connected, _, _| {
        let flex = match connected {
            true => Flex::row()
                    .with_child(view_tree().lens(State::tree).border(Color::BLACK, 0.5))
                    .with_spacer(5.0)
                    .with_flex_child(view_main(), 90.0),
            false => Flex::column()
                .with_child(Label::new("Please enter the ip address of the application to debug:"))
                .with_spacer(15.0)
                .with_child(TextBox::new().lens(State::address))
                .with_spacer(5.0)
                .with_child(Button::new("Connect").on_click(handle_connect).padding(5.0))
        };
        Box::new(Padding::new(10.0, Flex::column()
            .with_flex_child(Align::centered(flex.expand()), 90.0)
            .with_flex_child(Align::vertical(UnitPoint::BOTTOM, Label::dynamic(|data: &State, _| data.status.clone())), 5.0)))
    })
}
