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

use druid::{Color, LensExt, Widget, WidgetExt, WindowDesc};
use druid::im::Vector;
use druid::widget::{Button, Flex, Label, Padding, ViewSwitcher};
use druid_widget_nursery::ListSelect;
use crate::command::SPAWN_WINDOW;
use crate::state::{SpanLogEntry, State, StateHistory};
use crate::view::common;
use crate::view::menu::build_basic_menu;
use crate::window::{Destroy, Window};
use crate::window::events_window::EventsWindow;

fn view_active_start() -> impl Widget<StateHistory> {
    let duration = ViewSwitcher::new(|data: &StateHistory, _| data.selected_history.duration,
                                     |duration, _, _| Box::new(Label::new(format!("{}s", duration))));
    let values = ViewSwitcher::new(
        |data: &StateHistory, _| data.selected_history.values.clone(),
        |values, _, _| Box::new(common::build_values_view(values.iter())));

    let basic = common::build_box("Duration").with_child(duration);
    let values = common::build_box("Values").with_child(values);
    Flex::column()
        .with_child(basic.border(Color::BLACK, 0.5))
        .with_spacer(10.0)
        .with_child(values.border(Color::BLACK, 0.5))
}

fn view_actions(window: usize) -> impl Widget<State> {
    Flex::row()
        .with_child(
            Button::new("View events")
                .on_click(move |ctx, data: &mut State, _| {
                    let events = data.history_windows[window].selected_history.events.clone();
                    if let Some(window) = EventsWindow::new(data, events) {
                        ctx.submit_command(SPAWN_WINDOW.with(Box::new(window)));
                    }
                })
        )
}

fn view_active(window: usize) -> impl Widget<State> {
    Flex::column()
        .with_child(view_active_start().lens(State::history_windows.index(window)))
        .with_spacer(10.0)
        .with_child(view_actions(window))
}

fn list() -> impl Widget<StateHistory> {
    ViewSwitcher::new(
        |data: &StateHistory, _| data.history.clone(),
        |history, _, _|
            Box::new(
                ListSelect::new(history.iter()
                    .map(|v| (format!("{}", v.duration), v.clone())))
                    .scroll()
                    .lens(StateHistory::selected_history)
            )
    )
}

fn history_view(window: usize) -> impl Widget<State> {
    Flex::row()
        .with_child(list().lens(State::history_windows.index(window)))
        .with_spacer(5.0)
        .with_flex_child(
            Padding::new(20.0, view_active(window))
                .scroll()
                .center()
                .expand()
                .border(Color::BLACK, 0.5),
            90.0
        )
}

fn history_window(window: usize) -> impl Widget<State> {
    Padding::new(10.0, history_view(window))
}

pub struct Destructor(usize);

impl Destroy for Destructor {
    fn destroy(&self, state: &mut State) {
        state.history_windows.remove(self.0);
    }
}

pub struct HistoryWindow(usize);

impl HistoryWindow {
    pub fn new(state: &mut State, history: Vector<SpanLogEntry>) -> Option<Self> {
        if let Some(item) = history.iter().nth(0) {
            let handle = state.history_windows.insert(StateHistory {
                selected_history: item.clone(),
                history
            });
            Some(Self(handle))
        } else {
            state.status = "No history is available for this node at this time.".into();
            None
        }
    }
}

impl Window for HistoryWindow {
    fn build(&self) -> WindowDesc<State> {
        WindowDesc::new(history_window(self.0)).title("History").menu(build_basic_menu)
    }

    fn destructor(&self) -> Option<Box<dyn Destroy>> {
        Some(Box::new(Destructor(self.0)))
    }
}
