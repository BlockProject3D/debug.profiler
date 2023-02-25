// Copyright (c) 2023, BlockProject 3D
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

package com.github.blockproject3d.profiler.network.message.payload;

import com.github.blockproject3d.profiler.network.message.CompoundPayloadMessage;
import com.github.blockproject3d.profiler.network.message.component.I64;
import com.github.blockproject3d.profiler.network.message.component.LevelHeader;
import com.github.blockproject3d.profiler.network.message.component.U32;
import com.github.blockproject3d.profiler.network.message.component.Vchar;

public class SpanEvent extends CompoundPayloadMessage {
    private final U32 id = new U32();
    private final I64 timestamp = new I64();
    private final LevelHeader level = new LevelHeader();
    private final Vchar message = new Vchar();

    public SpanEvent() {
        add(id);
        add(timestamp);
        add(level);
        add(message);
    }

    public long getId() {
        return id.getValue();
    }

    public long getTimestamp() {
        return timestamp.getValue();
    }

    public LevelHeader.Level getLevel() {
        return level.getLevel();
    }

    public String getMessage() {
        return message.getData();
    }

    @Override
    public boolean isTerminate() {
        return false;
    }
}
