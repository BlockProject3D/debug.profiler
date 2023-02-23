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

import com.github.blockproject3d.profiler.network.message.IMessage;
import com.github.blockproject3d.profiler.network.protocol.ProtocolMismatchException;
import com.github.blockproject3d.profiler.network.protocol.SignatureMismatchException;
import com.github.blockproject3d.profiler.network.protocol.VersionMismatchException;
import org.slf4j.Logger;
import org.slf4j.LoggerFactory;

import java.io.DataInputStream;
import java.io.IOException;
import java.net.Socket;
import java.util.concurrent.ArrayBlockingQueue;

public class NetworkManager implements Runnable {
    private static final Logger LOGGER = LoggerFactory.getLogger(NetworkManager.class);

    private final ArrayBlockingQueue<IMessage> messageQueue = new ArrayBlockingQueue<>(Constants.MESSAGE_QUEUE_SIZE);
    private final String ip;
    private final int port;

    public NetworkManager(String ip, int port) {
        this.ip = ip;
        this.port = port;
        new Thread(this).start();
    }

    public NetworkManager(String ip) {
        this.ip = ip;
        this.port = Constants.DEFAULT_PORT;
        new Thread(this).start();
    }

    public IMessage pollMessage() {
        return messageQueue.poll();
    }

    private Socket connect() {
        try {
            Socket socket = new Socket(ip, port);
            Constants.PROTOCOL.initialHandshake(socket.getInputStream(), socket.getOutputStream());
            return socket;
        } catch (IOException | SignatureMismatchException | ProtocolMismatchException | VersionMismatchException e) {
            LOGGER.error("Failed to connect to server", e);
            return null;
        }
    }

    private IMessage readMessage(DataInputStream stream) throws IOException {
        byte type = stream.readByte();
        IMessage msg = MessageRegistry.get(type);
        if (msg == null)
            return null;
        byte[] header = new byte[msg.getHeaderSize()];
        stream.readFully(header);
        msg.loadHeader(header, 0);
        if (msg.getPayloadSize() > 0) {
            byte[] payload = new byte[msg.getPayloadSize()];
            stream.readFully(payload);
            msg.loadPayload(payload, 0);
        }
        return msg;
    }

    @Override
    public void run() {
        Socket socket = connect();
        if (socket != null) {
            try {
                DataInputStream stream = new DataInputStream(socket.getInputStream());
                boolean terminate_msg = false;
                while (!terminate_msg) {
                    IMessage msg = readMessage(stream);
                    if (msg != null) {
                        if (msg.isTerminate())
                            terminate_msg = true;
                        LOGGER.debug("New message loaded: {}", msg);
                        messageQueue.put(msg);
                    }
                }
            } catch (IOException | InterruptedException e) {
                LOGGER.error("Failed to read server message", e);
            }
            try {
                socket.close();
            } catch (IOException e) {
                LOGGER.error("Failed to close socket", e);
            }
        }
    }
}
