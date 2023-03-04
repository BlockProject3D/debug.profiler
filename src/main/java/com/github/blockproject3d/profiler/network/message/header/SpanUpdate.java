package com.github.blockproject3d.profiler.network.message.header;

import com.github.blockproject3d.profiler.network.message.CompoundMessage;
import com.github.blockproject3d.profiler.network.message.component.U32;

public class SpanUpdate extends CompoundMessage {
    private final U32 id = new U32();
    private final U32 runCount = new U32();
    private final Duration averageTime = new Duration();
    private final Duration minTime = new Duration();
    private final Duration maxTime = new Duration();

    public SpanUpdate() {
        add(id);
        add(runCount);
        add(averageTime);
        add(minTime);
        add(maxTime);
    }

    public long getId() {
        return id.getValue();
    }

    public long getRunCount() {
        return runCount.getValue();
    }

    public Duration getAverageTime() {
        return averageTime;
    }

    public Duration getMinTime() {
        return minTime;
    }

    public Duration getMaxTime() {
        return maxTime;
    }

    @Override
    public boolean isTerminate() {
        return false;
    }
}
