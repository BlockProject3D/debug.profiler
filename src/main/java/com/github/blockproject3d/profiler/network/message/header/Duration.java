package com.github.blockproject3d.profiler.network.message.header;

import com.github.blockproject3d.profiler.network.message.CompoundMessage;
import com.github.blockproject3d.profiler.network.message.component.U32;

public class Duration extends CompoundMessage {
    private final U32 seconds = new U32();
    private final U32 nanoSeconds = new U32();

    public Duration() {
        add(seconds);
        add(nanoSeconds);
    }

    public long getSeconds() {
        return seconds.getValue();
    }

    public int getMicroSeconds() {
        long nanos = nanoSeconds.getValue();
        nanos -= (nanos / 1000 / 1000) * 1000000;
        return (int)(nanos / 1000);
    }

    public int getMilliSeconds() {
        long nanos = nanoSeconds.getValue();
        nanos -= (long)getMicroSeconds() * 1000;
        return (int)(nanos / 1000 / 1000);
    }

    @Override
    public boolean isTerminate() {
        return false;
    }
}
