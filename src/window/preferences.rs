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

use druid::{LensExt, Widget, WidgetExt, WindowDesc};
use druid::text::ParseFormatter;
use druid::widget::{Checkbox, Flex, Label, TextBox, ValueTextBox};
use druid_widget_nursery::ListSelect;
use crate::state::{Preferences, State, Theme};
use crate::view::menu::build_basic_menu;
use crate::Window;
use crate::window::Destroy;

fn preferences_window() -> impl Widget<State> {
    Flex::column()
        .with_child(
            Flex::row()
                .with_child(Label::new("Maximum history (0 = unlimited): "))
                .with_child(
                    ValueTextBox::new(TextBox::new(), ParseFormatter::new())
                        .lens(State::preferences.then(Preferences::max_history))
                )
        )
        .with_spacer(10.0)
        .with_child(
            Flex::row()
                .with_child(Label::new("Maximum events (0 = unlimited): "))
                .with_child(
                    ValueTextBox::new(TextBox::new(), ParseFormatter::new())
                        .lens(State::preferences.then(Preferences::max_events))
                )
        )
        .with_spacer(10.0)
        .with_child(
            Flex::row()
                .with_child(Label::new("Theme variant: "))
                .with_child(
                    ListSelect::new(vec![
                        ("Light", Theme::Light),
                        ("Dark", Theme::Dark)
                    ]).lens(State::preferences.then(Preferences::theme))
                )
        )
        .with_spacer(10.0)
        .with_child(
            Checkbox::new("Inherit variables from parent")
                .lens(State::preferences.then(Preferences::inherit))
        )
        .center()
}

pub struct PreferencesWindow;

impl Window for PreferencesWindow {
    fn build(&self) -> WindowDesc<State> {
        WindowDesc::new(preferences_window()).menu(build_basic_menu).title("Preferences")
    }

    fn destructor(&self) -> Option<Box<dyn Destroy>> {
        None
    }
}