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
use druid::{Color, LensExt, Widget, WidgetExt, WindowDesc};
use druid::im::Vector;
use druid::widget::{Flex, Padding, ViewSwitcher};
use druid_widget_nursery::ListSelect;
use crate::state::{Event, State, StateEvents};
use crate::view::common;
use crate::window::{Destroy, Window};

fn events_view() -> impl Widget<StateEvents> {
    let list = ViewSwitcher::new(
        |data: &StateEvents, _| data.events.clone(),
        |events, _, _| {
            Box::new(
                ListSelect::new(events.iter().map(|v| (v.msg.clone(), v.clone())))
                    .scroll()
                    .lens(StateEvents::selected_event)
            )
        });
    let values = ViewSwitcher::new(
        |data: &StateEvents, _| data.selected_event.clone(),
        |event, _, _| Box::new(common::build_values_view(event.values.iter().map(|(s, v)| (s, v)))));

    let values = common::build_box("Values")
        .with_child(values)
        .scroll();
    Flex::column()
        .with_flex_child(list.expand().border(Color::BLACK, 0.5), 50.0)
        .with_flex_child(values.center().expand().border(Color::BLACK, 0.5), 50.0)
}

fn events_window(window: usize) -> impl Widget<State> {
    Padding::new(10.0, events_view().lens(State::event_windows.index(window)))
}

struct Destructor(usize);

impl Destroy for Destructor {
    fn destroy(&self, state: &mut State) {
        state.event_windows.remove(self.0)
    }
}

pub struct EventsWindow(usize);

impl EventsWindow {
    pub fn new(state: &mut State, events: Vector<Arc<Event>>) -> Option<Self> {
        if let Some(event) = events.iter().nth(0) {
            let handle = state.event_windows.insert(StateEvents {
                selected_event: event.clone(),
                events,
            });
            Some(Self(handle))
        } else {
            state.status = "No events are available for this node at this time.".into();
            None
        }
    }
}

impl Window for EventsWindow {
    fn build(&self) -> WindowDesc<State> {
        WindowDesc::new(events_window(self.0)).title("Events")
    }

    fn destructor(&self) -> Option<Box<dyn Destroy>> {
        Some(Box::new(Destructor(self.0)))
    }
}
