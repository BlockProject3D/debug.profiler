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

use std::ops::{Index, IndexMut};
use druid::Data;
use druid::im::Vector;

// This module contains a hacky container to bypass one of the WORST and most stupid API design in
// im-rs crate: HashMap index function only accepts static references!

#[derive(Data, Clone)]
pub struct WindowMap<T>(Vector<Option<T>>);

impl<T: Clone> Default for WindowMap<T> {
    fn default() -> Self {
        Self(Vector::new())
    }
}

impl<T: Clone> Index<usize> for WindowMap<T> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        self.0[index].as_ref().unwrap()
    }
}

impl<T: Clone> IndexMut<usize> for WindowMap<T> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        self.0[index].as_mut().unwrap()
    }
}

impl<T: Clone> WindowMap<T> {
    pub fn insert(&mut self, window: T) -> usize {
        for (i, v) in self.0.iter_mut().enumerate() {
            if v.is_none() {
                *v = Some(window);
                return i;
            }
        }
        self.0.push_back(Some(window));
        self.0.len() - 1
    }

    pub fn remove(&mut self, index: usize) {
        if index > self.0.len() {
            return;
        }
        let mut flag = true;
        for i in index..self.0.len() {
            if self.0[i].is_some() {
                flag = false;
            }
        }
        if flag { //It is safe to remove the object from the vector.
            self.0.remove(index);
        } else { //Removing the object from the vector would cause dangling indices.
            self.0[index] = None;
        }
    }
}
