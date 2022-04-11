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

use druid::{Color, Data, FontDescriptor, FontFamily, FontWeight, Widget};
use druid::widget::{Flex, Label};
use crate::network_types::Value;

pub fn bold_font() -> FontDescriptor {
    FontDescriptor::new(FontFamily::SYSTEM_UI)
        .with_weight(FontWeight::BOLD)
        .with_size(20.0)
}

pub fn small_bold_font() -> FontDescriptor {
    FontDescriptor::new(FontFamily::SYSTEM_UI)
        .with_size(16.0)
        .with_weight(FontWeight::BOLD)
}

pub fn build_box<T: Data>(name: &str) -> Flex<T> {
    let font = bold_font();
    Flex::column()
        .with_child(Label::new(name).with_font(font.clone()))
        .with_spacer(5.0)
}

pub fn build_bool_view<T: Data>(val: bool) -> impl Widget<T> {
    match val {
        true => Label::new("Yes").with_text_color(Color::rgb8(0, 255, 0)),
        false => Label::new("No").with_text_color(Color::rgb8(255, 0, 0))
    }
}

//Yay rust is buggy broken: using impl Widget makes it reject it, but Flex<T> works wtf!!
pub fn build_values_view<'a, T: Data>(iter: impl Iterator<Item = (&'a String, &'a Value)>) -> Flex<T> {
    let name_font = small_bold_font();
    let mut flex = Flex::column();
    for (name, value) in iter {
        flex.add_child(
            Flex::row()
                .with_child(Label::new(name.clone()).with_font(name_font.clone()))
                .with_spacer(15.5)
                .with_child(Label::new(value.to_string()))
        );
    }
    flex
}
