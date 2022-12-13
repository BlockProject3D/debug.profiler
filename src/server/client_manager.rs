// Copyright (c) 2022, BlockProject 3D
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

use super::client::{Client, ClientTaskResult};
use futures::{
    stream::{FuturesUnordered, Next},
    StreamExt,
};
use std::{
    future::Future,
    io::Result,
    pin::Pin,
    task::{Context, Poll},
};
use tokio::task::JoinError;

pub type JoinResult<T> = std::result::Result<T, JoinError>;

struct DataHack<T: Clone, F: Future> {
    user_data: T,
    future: F
}

impl<T: Clone, F: Future> DataHack<T, F> {
    pub fn new(user_data: T, future: F) -> DataHack<T, F> {
        DataHack { user_data, future }
    }
}

impl<T: Clone, F: Future> Future for DataHack<T, F> {
    type Output = (T, F::Output);

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        unsafe {
            let raw = self.get_unchecked_mut();
            let pin = Pin::new_unchecked(&mut raw.future);
            match pin.poll(cx) {
                Poll::Ready(v) => Poll::Ready((raw.user_data.clone(), v)),
                Poll::Pending => Poll::Pending
            }
        }
    }
}

struct TaskList<'a, T: Future>(Next<'a, FuturesUnordered<T>>);

impl<'a, T: Future> Future for TaskList<'a, T> {
    type Output = T::Output;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let pin = Pin::new(&mut self.0);
        if let Poll::Ready(v) = pin.poll(cx) {
            match v {
                Some(v) => Poll::Ready(v),
                None => Poll::Pending,
            }
        } else {
            Poll::Pending
        }
    }
}

pub struct ClientManager {
    clients: Vec<Client>,
    tasks: FuturesUnordered<DataHack<usize, ClientTaskResult>>,
    cur_index: usize,
}

impl ClientManager {
    pub fn new() -> ClientManager {
        ClientManager {
            clients: Vec::new(),
            tasks: FuturesUnordered::new(),
            cur_index: 0,
        }
    }

    pub fn add(&mut self, connection_string: String) {
        let (client, task) = Client::new(connection_string, self.cur_index);
        self.cur_index += 1;
        self.tasks.push(DataHack::new(client.index(), task));
        self.clients.push(client);
    }

    pub async fn get_client_stop(&mut self) -> (usize, JoinResult<Result<()>>) {
        TaskList(self.tasks.next()).await
    }

    pub fn remove(&mut self, index: usize) {
        self.clients.retain(|v| v.index() != index)
    }

    pub fn get(&mut self, index: usize) -> Option<&mut Client> {
        self.clients.iter_mut().find(|v| v.index() == index)
    }

    pub async fn stop(mut self) {
        for mut v in self.clients {
            v.stop();
        }
        while let Some(_) = self.tasks.next().await {}
    }
}
