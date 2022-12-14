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

use crate::network_types as nt;
use crate::util::csv_format_single;

#[derive(Clone)]
pub struct ValueSet {
    data: Vec<(String, nt::Value)>,
}

impl From<Vec<(String, nt::Value)>> for ValueSet {
    fn from(data: Vec<(String, nt::Value)>) -> Self {
        ValueSet { data }
    }
}

impl ValueSet {
    pub fn inherit_from<T: IntoIterator<Item = (String, nt::Value)>>(
        &mut self,
        parent: &str,
        iter: T,
    ) {
        self.data.extend(
            iter.into_iter()
                .map(|(k, v)| (format!("{}::{}", parent, k), v)),
        )
    }

    pub fn to_string(self) -> String {
        let vals = self
            .data
            .into_iter()
            .map(|(k, v)| format!("{} = {}", k, v))
            .collect::<Vec<String>>();
        csv_format(vals.iter().map(|v| &**v))
    }

    pub fn into_iter(self) -> impl IntoIterator<Item = (String, nt::Value)> {
        self.data.into_iter()
    }
}

impl Extend<(String, nt::Value)> for ValueSet {
    fn extend<T: IntoIterator<Item = (String, nt::Value)>>(&mut self, iter: T) {
        self.data.extend(iter);
    }
}

pub fn csv_format<'a, T: IntoIterator<Item = &'a str>>(cols: T) -> String {
    cols.into_iter()
        .map(|v| csv_format_single(v, ','))
        .collect::<Vec<String>>()
        .join(",")
}
