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

use druid::{Color, LensExt, Widget, WidgetExt};
use druid::widget::{Button, Flex, Label, Padding, ViewSwitcher};
use druid_widget_nursery::ListSelect;
use crate::command::NODE_OPEN_EVENTS;
use crate::state::{State, StateHistory};

fn view_active() -> impl Widget<StateHistory> {
    let duration = ViewSwitcher::new(|data: &StateHistory, _| data.selected_history.duration,
                                     |duration, _, _| Box::new(Label::new(format!("{}s", duration))));
    let values = ViewSwitcher::new(
        |data: &StateHistory, _| data.selected_history.values.clone(),
        |values, _, _| Box::new(super::common::build_values_view(values.iter())));

    let basic = super::common::build_box("Duration").with_child(duration);
    let values = super::common::build_box("Values").with_child(values);
    let actions = Flex::row()
        .with_child(
            Button::new("View events")
                .on_click(|ctx, data: &mut StateHistory, _| {
                    let events = data.selected_history.events.clone();
                    ctx.submit_command(NODE_OPEN_EVENTS.with(events));
                })
        );
    Flex::column()
        .with_child(basic.border(Color::BLACK, 0.5))
        .with_spacer(10.0)
        .with_child(values.border(Color::BLACK, 0.5))
        .with_spacer(10.0)
        .with_child(Padding::new(5.0, actions).border(Color::BLACK, 0.5))
}

fn history_view() -> impl Widget<StateHistory> {
    let list = ViewSwitcher::new(|data: &StateHistory, _| data.history.clone(), |history, _, _| {
        Box::new(ListSelect::new(history.iter().map(|v| (format!("{}", v.duration), v.clone()))).scroll().lens(StateHistory::selected_history))
    });

    Flex::row()
        .with_child(list)
        .with_spacer(5.0)
        .with_flex_child(
            Padding::new(20.0, view_active())
                .scroll()
                .center()
                .expand()
                .border(Color::BLACK, 0.5),
            90.0
        )
}

pub fn history_window(window: usize) -> impl Widget<State> {
    Padding::new(10.0, history_view().lens(State::history_windows.index(window)))
}
