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

import java.io.IOException;
import java.io.InputStream;
import java.io.OutputStream;
import java.nio.charset.StandardCharsets;

public class Protocol {
    public final String name;
    public final Version version;

    public Protocol(String name, Version version) {
        this.name = name.toUpperCase();
        byte[] bytes = this.name.getBytes(StandardCharsets.UTF_8);
        if (bytes.length != 4)
            throw new IllegalArgumentException("The name of a BP3D protocol MUST always be 4 bytes");
        this.version = version;
    }

    public void initialHandshake(InputStream in, OutputStream out) throws IOException, VersionMismatchException, ProtocolMismatchException, SignatureMismatchException {
        byte[] buffer = new byte[40]; //The hello packet is always 40 bytes long.
        if (in.read(buffer) != 40 || buffer[0] != 'B' || buffer[1] != 'P' || buffer[2] != '3' || buffer[3] != 'D')
            throw new SignatureMismatchException();
        String name = new String(buffer, 4, 4, StandardCharsets.UTF_8);
        if (!name.equals(this.name))
            throw new ProtocolMismatchException(name, this.name);
        Version other = new Version(buffer, 8);
        if (!this.version.equals(other))
            throw new VersionMismatchException(other, this.version);
        this.version.getBytes(buffer, 8);
        out.write(buffer);
    }
}
