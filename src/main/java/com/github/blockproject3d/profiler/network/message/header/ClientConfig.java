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

package com.github.blockproject3d.profiler.network.message.header;

import com.github.blockproject3d.profiler.network.message.IWritable;
import com.github.blockproject3d.profiler.network.message.component.LevelHeader;
import com.github.blockproject3d.profiler.network.message.component.Option;
import com.github.blockproject3d.profiler.network.message.component.U32;

public class ClientConfig implements IWritable {
    private final U32 maxAveragePoints = new U32();
    private final U32 maxRows = new U32();
    private boolean canAccessPath;
    private final Option<LevelHeader> maxLevel =  new Option<>(null);

    public void setMaxAveragePoints(long points) {
        maxAveragePoints.setValue(points);
    }

    public void setMaxRows(long rows) {
        maxRows.setValue(rows);
    }

    public void setCanAccessPath(boolean flag) {
        canAccessPath = flag;
    }

    public void setMaxLevel(LevelHeader.Level level) {
        if (maxLevel.getValue() == null)
            maxLevel.setValue(new LevelHeader());
        maxLevel.getValue().setLevel(level);
    }

    public int getSize() {
        return 11;
    }

    @Override
    public int write(byte[] buffer, int offset) {
        int start = offset;
        offset += maxAveragePoints.write(buffer, offset);
        offset += maxRows.write(buffer, offset);
        buffer[offset] = canAccessPath ? (byte)1 : (byte)0;
        offset += 1;
        offset += maxLevel.write(buffer, offset);
        return offset - start;
    }
}
