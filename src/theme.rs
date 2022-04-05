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

use druid::{Color, Env, Key};
use druid::theme::{BACKGROUND_DARK, BACKGROUND_LIGHT, BORDER_DARK, BORDER_LIGHT, BUTTON_DARK, BUTTON_LIGHT, CURSOR_COLOR, DISABLED_BUTTON_DARK, DISABLED_BUTTON_LIGHT, DISABLED_FOREGROUND_DARK, DISABLED_FOREGROUND_LIGHT, DISABLED_TEXT_COLOR, FOREGROUND_DARK, FOREGROUND_LIGHT, PLACEHOLDER_COLOR, PRIMARY_DARK, PRIMARY_LIGHT, SCROLLBAR_BORDER_COLOR, SCROLLBAR_COLOR, SELECTED_TEXT_BACKGROUND_COLOR, SELECTED_TEXT_INACTIVE_BACKGROUND_COLOR, SELECTION_TEXT_COLOR, TEXT_COLOR, WINDOW_BACKGROUND_COLOR};
use crate::state::State;

fn invert_color(key: &Key<Color>, env: &mut Env) {
    let color = env.get(key);
    let (r, g, b, a) = color.as_rgba8();
    // The slower it gets the better it is!
    // druid asks for clone well give it worse performance!!
    env.set(key.clone(), Color::rgba8(0xFF - r, 0xFF - g, 0xFF - b, a));
}

const COLOR_KEYS: &[Key<Color>] = &[WINDOW_BACKGROUND_COLOR, TEXT_COLOR, DISABLED_TEXT_COLOR,
    PLACEHOLDER_COLOR, PRIMARY_LIGHT, PRIMARY_DARK, BACKGROUND_LIGHT, BACKGROUND_DARK,
    FOREGROUND_LIGHT, FOREGROUND_DARK, DISABLED_FOREGROUND_LIGHT, DISABLED_FOREGROUND_DARK,
    BUTTON_DARK, BUTTON_LIGHT, DISABLED_BUTTON_DARK, DISABLED_BUTTON_LIGHT, BORDER_DARK,
    BORDER_LIGHT, SELECTED_TEXT_BACKGROUND_COLOR, SELECTED_TEXT_INACTIVE_BACKGROUND_COLOR,
    SELECTION_TEXT_COLOR, CURSOR_COLOR, SCROLLBAR_COLOR, SCROLLBAR_BORDER_COLOR
];

pub fn overwrite_theme(env: &mut Env, _: &State) {
    for key in COLOR_KEYS {
        invert_color(key, env);
    }
    env.set(WINDOW_BACKGROUND_COLOR, Color::rgb8(240, 240, 240));
}
