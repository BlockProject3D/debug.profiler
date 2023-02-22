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

package com.github.blockproject3d.profiler.network.message;

import java.util.ArrayList;

public abstract class CompoundMessage implements IMessage {
    protected ArrayList<IMessage> components = new ArrayList<>();

    protected abstract void load();

    @Override
    public int getHeaderSize() {
        int len = 0;
        for (IMessage component: components) {
            len += component.getHeaderSize();
        }
        return len;
    }

    @Override
    public int getPayloadSize() {
        int len = 0;
        for (IMessage component: components) {
            len += component.getPayloadSize();
        }
        return len;
    }

    @Override
    public void loadHeader(byte[] header, int offset) {
        for (IMessage component: components) {
            component.loadHeader(header, offset);
            offset += component.getHeaderSize();
        }
        if (getPayloadSize() <= 0)
            load();
    }

    @Override
    public void loadPayload(byte[] payload, int offset) {
        for (IMessage component: components) {
            component.loadPayload(payload, offset);
            offset += component.getPayloadSize();
        }
        load();
    }
}