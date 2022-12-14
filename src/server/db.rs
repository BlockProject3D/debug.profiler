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

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use bp3d_threads::{Atomic, AtomicCell};
use crate::network_types as nt;

impl Atomic for nt::Duration {
    type Atomic = AtomicU64;

    fn atomic_new(value: Self) -> Self::Atomic {
        let garbagefuckingrust = value.seconds as u64;
        let val = (garbagefuckingrust << 32) + value.nano_seconds as u64;
        AtomicU64::new(val)
    }

    fn atomic_set(atomic: &Self::Atomic, value: Self) {
        let garbagefuckingrust = value.seconds as u64;
        let val = (garbagefuckingrust << 32) + value.nano_seconds as u64;
        atomic.store(val, Ordering::Release);
    }

    fn atomic_get(atomic: &Self::Atomic) -> Self {
        let val = atomic.load(Ordering::Acquire);
        let seconds = (val >> 32) as u32;
        let nano_seconds = (val & 0xFFFFFFFF) as u32;
        nt::Duration {
            seconds,
            nano_seconds
        }
    }
}

#[derive(Clone)]
pub struct SpanSnapshot {
    pub metadata: Arc<nt::Metadata>,
    pub min: AtomicCell<nt::Duration>,
    pub max: AtomicCell<nt::Duration>,
    pub average: AtomicCell<nt::Duration>,
    pub run_count: AtomicCell<usize>,
    pub is_dropped: AtomicCell<bool>,
    pub is_active: AtomicCell<bool>
}

struct Inner {
    snapshots: HashMap<u32, Arc<SpanSnapshot>>,
    list: Vec<String>
}

#[derive(Clone)]
pub struct SpanStore {
    inner: Arc<Mutex<Inner>>
}

impl SpanStore {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(Inner {
                snapshots: HashMap::new(),
                list: Vec::new()
            }))
        }
    }

    pub fn alloc_span(&self, id: u32, metadata: Arc<nt::Metadata>) -> Arc<SpanSnapshot> {
        let mut lock = self.inner.lock().unwrap();
        let ptr = Arc::new(SpanSnapshot {
            metadata,
            min: AtomicCell::new(Duration::new(0, 0).into()),
            max: AtomicCell::new(Duration::new(0, 0).into()),
            average: AtomicCell::new(Duration::new(0, 0).into()),
            run_count: AtomicCell::new(0),
            is_active: AtomicCell::new(false),
            is_dropped: AtomicCell::new(false)
        });
        lock.snapshots.insert(id, ptr.clone());
        ptr
    }
}
