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

use druid::{Color, Widget, WidgetExt, WindowDesc};
use druid::widget::{BackgroundBrush, Flex, Label};
use crate::constants::APP_NAME;
use crate::state::State;
use crate::view::common::bold_font;
use crate::view::menu::build_basic_menu;
use crate::window::{Destroy, Window};

const ABOUT_LINE1: &str = "BP3D Profiler, Copyright 2022 BlockProject 3D";
const ABOUT_LINE2: &str = "BSD-3-Clause License";

const LICENSE: &str = r#"Copyright (c) 2022, BlockProject 3D

All rights reserved.

Redistribution and use in source and binary forms, with or without modification,
are permitted provided that the following conditions are met:

    * Redistributions of source code must retain the above copyright notice,
      this list of conditions and the following disclaimer.
    * Redistributions in binary form must reproduce the above copyright notice,
      this list of conditions and the following disclaimer in the documentation
      and/or other materials provided with the distribution.
    * Neither the name of BlockProject 3D nor the names of its contributors
      may be used to endorse or promote products derived from this software
      without specific prior written permission.

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
"AS IS" AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR
A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT OWNER OR
CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL,
EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO,
PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR
PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF
LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING
NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS
SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE."#;

fn view() -> impl Widget<State> {
    let namev = format!("{}  |  {}", APP_NAME, env!("CARGO_PKG_VERSION"));
    Flex::column()
        .with_flex_child(Label::new(namev).with_font(bold_font()), 1.0)
        .with_spacer(30.0)
        .with_flex_child(Label::new(ABOUT_LINE1), 1.0)
        .with_spacer(20.0)
        .with_flex_child(Label::new(ABOUT_LINE2), 1.0)
        .with_spacer(7.0)
        .with_flex_child(
            Label::new(LICENSE)
                .with_text_color(Color::BLACK)
                .padding(10.0)
                .background(BackgroundBrush::Color(Color::WHITE))
                .scroll(),
            15.0
        )
        .center()
        .padding(10.0)
}

pub struct AboutWindow;

impl Window for AboutWindow {
    fn build(&self) -> WindowDesc<State> {
        WindowDesc::new(view()).title("About BP3D Profiler").menu(build_basic_menu)
            .window_size((705.0, 400.0))
    }

    fn destructor(&self) -> Option<Box<dyn Destroy>> {
        None
    }
}
