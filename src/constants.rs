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

use std::time::Duration;
use druid::Color;

//
// --> Basic constants <--
//

/// The name of the application in the file system.
pub const FILESYS_APP_NAME: &str = "bp3d-profiler";

/// The name of the application for display.
pub const APP_NAME: &str = "BP3D Profiler";

//
// --> Network constants <--
//

/// Timeout for all network reads.
pub const NET_READ_DURATION: Duration = Duration::from_millis(500);

/// Default size of fast forward command buffer (512 in debug and 8192 in release).
///
/// This is set to 512 in debug because druid is atrociously slow in that configuration.
#[cfg(debug_assertions)]
pub const DEFAULT_MAX_SUB_BUFFER: usize = 512;

/// Default size of the buffer to store bytes of a single command.
pub const DEFAULT_SINGLE_COMMAND_BUFFER: usize = 256;

/// Default size of fast forward command buffer (512 in debug and 8192 in release).
///
/// This is set to 512 in debug because druid is atrociously slow in that configuration.
#[cfg(not(debug_assertions))]
pub const DEFAULT_MAX_SUB_BUFFER: usize = 8192;

/// The multiplier to multiply to the size of the fast forward command buffer to obtain the size of
/// the main command buffer and channel.
pub const MAX_BUFFER_MULTIPLIER: usize = 2;

/// Version of UDP auto-discovery protocol.
pub const AUTODISCOVERY_PROTOCOL_VERSION: u8 = 0;

/// The default port for both UDP (auto discovery) and TCP.
pub const DEFAULT_PORT: u16 = 4026;

//
// --> GUI constants <--
//

/// The color for Level::TRACE.
pub const COLOR_TRACE: Color = Color::rgb8(0, 255, 255);

/// The color for Level::DEBUG.
pub const COLOR_DEBUG: Color = Color::rgb8(0, 0, 255);

/// The color for Level::INFO.
pub const COLOR_INFO: Color = Color::rgb8(0, 255, 0);

/// The color for Level::WARN.
pub const COLOR_WARN: Color = Color::rgb8(255, 255, 0);

/// The color for Level::ERROR.
pub const COLOR_ERR: Color = Color::rgb8(255, 0, 0);
