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

package com.github.blockproject3d.profiler.network;

import com.github.blockproject3d.profiler.network.message.*;
import com.github.blockproject3d.profiler.network.message.header.SpanFollows;
import com.github.blockproject3d.profiler.network.message.header.SpanParent;
import com.github.blockproject3d.profiler.network.message.header.SpanUpdate;
import com.github.blockproject3d.profiler.network.message.payload.Project;
import com.github.blockproject3d.profiler.network.message.payload.SpanAlloc;
import com.github.blockproject3d.profiler.network.message.payload.SpanEvent;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.lang.reflect.InvocationTargetException;
import java.util.HashMap;

public class MessageRegistry {
    private static final Logger LOGGER = LoggerFactory.getLogger(MessageRegistry.class);

    private static final HashMap<Byte, Class<? extends IMessage>> REGISTRY = new HashMap<>();

    public static void register(int type, Class<? extends IMessage> msgClass) {
        if (REGISTRY.containsKey((byte)type))
            throw new ArrayStoreException("The message type '" + (byte)type + "' is already registered");
        REGISTRY.put((byte)type, msgClass);
    }

    public static IMessage get(byte type) {
        if (!REGISTRY.containsKey(type)) {
            LOGGER.error("Unknown message type '{}'", type);
            return null;
        }
        LOGGER.debug("Instantiating message with type '{}'", type);
        try {
            return REGISTRY.get(type).getDeclaredConstructor().newInstance();
        } catch (InstantiationException | IllegalAccessException | InvocationTargetException | NoSuchMethodException e) {
            LOGGER.error("Failed to instantiate message", e);
            return null;
        }
    }

    static {
        register(0, Project.class);
        register(1, SpanAlloc.class);
        register(2, SpanParent.class);
        register(3, SpanFollows.class);
        register(4, SpanEvent.class);
        register(5, SpanUpdate.class);
    }
}
