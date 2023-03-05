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

package com.github.blockproject3d.profiler.network.message.component;

import com.github.blockproject3d.profiler.network.message.IHeaderComponent;
import com.github.blockproject3d.profiler.network.message.IPayloadComponent;
import com.github.blockproject3d.profiler.network.message.IWritable;

public class Option<T extends IHeaderComponent> implements IHeaderComponent, IPayloadComponent, IWritable {
    private T msg;
    private boolean isValid = false;

    public Option(T msg) {
        this.msg = msg;
    }

    public T getValue() {
        return isValid ? msg : null;
    }

    public void setValue(T value) {
        isValid = value != null;
        msg = value;
    }

    @Override
    public int getHeaderSize() {
        return 1 + msg.getHeaderSize();
    }

    @Override
    public int getPayloadSize() {
        if (msg instanceof IPayloadComponent) {
            return !isValid ? 0 : ((IPayloadComponent) msg).getPayloadSize();
        }
        return 0;
    }

    @Override
    public void loadHeader(byte[] header, int offset) {
        if (header[offset] == 1) {
            msg.loadHeader(header, offset + 1);
            isValid = true;
        }
    }

    @Override
    public void loadPayload(byte[] payload, int offset) {
        if (msg instanceof IPayloadComponent) {
            if (isValid)
                ((IPayloadComponent) msg).loadPayload(payload, offset);
        }
    }

    @Override
    public int write(byte[] buffer, int offset) {
        if (isValid)
            buffer[offset] = 1;
        else
            buffer[offset] = 0;
        if (msg != null && msg instanceof IWritable)
            return 1 + ((IWritable) msg).write(buffer, offset + 1);
        return 1;
    }
}
