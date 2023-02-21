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

package com.github.blockproject3d.profiler.network.protocol;

import java.nio.ByteBuffer;
import java.nio.ByteOrder;
import java.nio.charset.StandardCharsets;
import java.util.Objects;

public class Version {
    public final String preRelease;
    public final long major;

    public Version(long major) {
        this.major = major;
        this.preRelease = null;
    }

    public Version(long major, String preRelease) {
        this.major = major;
        this.preRelease = preRelease;
    }

    public Version(byte[] buffer, int offset) {
        //the major version is a u64.
        this.major = ByteBuffer.wrap(buffer, offset, 8).order(ByteOrder.LITTLE_ENDIAN).getLong();
        //24 bytes for the pre-release string
        int len = 0;
        while (len < 24 && buffer[offset + 8 + len] != 0x0)
            ++len;
        this.preRelease = len == 0 ? null : new String(buffer, offset + 8, len, StandardCharsets.UTF_8);
    }

    public void getBytes(byte[] buffer, int offset) {
        ByteBuffer.wrap(buffer, offset, 8).order(ByteOrder.LITTLE_ENDIAN).putLong(this.major);
        for (int i = 0; i != 24; ++i) {
            buffer[offset + 8 + i] = 0;
        }
        if (this.preRelease != null) {
            byte[] bytes = this.preRelease.getBytes(StandardCharsets.UTF_8);
            int len = Math.min(bytes.length, 24);
            for (int i = 0; i != len; ++i) {
                buffer[i + offset + 8 + i] = bytes[i];
            }
        }
    }

    @Override
    public String toString() {
        return preRelease != null ? major + "-" + preRelease : String.valueOf(major);
    }

    @Override
    public boolean equals(Object o) {
        if (this == o)
            return true;
        if (o == null || getClass() != o.getClass())
            return false;
        Version other = (Version)o;
        return major == other.major && Objects.equals(preRelease, other.preRelease);
    }

    @Override
    public int hashCode() {
        return Objects.hash(preRelease, major);
    }
}
