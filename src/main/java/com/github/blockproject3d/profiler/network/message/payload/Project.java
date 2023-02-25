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
import com.github.blockproject3d.profiler.network.message.component.Option;
import com.github.blockproject3d.profiler.network.message.component.Vchar;

public class Project extends CompoundPayloadMessage {
    private final Vchar appName = new Vchar();
    private final Vchar name = new Vchar();
    private final Vchar version = new Vchar();
    private final Vchar commandLine = new Vchar();

    private final Target target = new Target();
    private final Option<Cpu> cpu = new Option<>(new Cpu());

    public Project() {
        add(appName);
        add(name);
        add(version);
        add(commandLine);
        add(target);
        add(cpu);
    }

    public String getAppName() {
        return appName.getData();
    }

    public String getName() {
        return name.getData();
    }

    public String getVersion() {
        return version.getData();
    }

    public String getCommandLine() {
        return commandLine.getData();
    }

    public Target getTarget() {
        return target;
    }

    public Cpu getCpu() {
        return cpu.getValue();
    }

    @Override
    public boolean isTerminate() {
        return false;
    }
}
