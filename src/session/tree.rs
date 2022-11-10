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

use std::{collections::VecDeque, io::Result, sync::Arc};

use tokio::io::{AsyncWrite, AsyncWriteExt};

use crate::network_types::Metadata;

#[derive(Clone, Debug)]
pub struct Span {
    pub metadata: Arc<Metadata>,
    pub id: u32,
    pub expanded: bool,
    children: Vec<Span>,
}

impl Span {
    pub fn new() -> Span {
        Span {
            metadata: Arc::new(Metadata::default()),
            id: 0,
            expanded: true,
            children: Vec::new(),
        }
    }

    pub fn with_metadata(id: u32, metadata: Arc<Metadata>) -> Span {
        Span {
            metadata,
            id,
            expanded: true,
            children: Vec::new(),
        }
    }

    /// Attempts to find the parent of the specified node.
    pub fn find_parent(&self, id: u32) -> Option<u32> {
        for v in &self.children {
            if v.id == id {
                return Some(self.id);
            }
            if let Some(id) = v.find_parent(id) {
                return Some(id);
            }
        }
        None
    }

    /// Attempts to remove the specified node.
    ///
    /// If the node wasn't found, None is returned.
    /// If the node was found and removed, the removed node is returned.
    pub fn remove_node(&mut self, id: u32) -> Option<Span> {
        let index =
            self.children
                .iter()
                .enumerate()
                .find_map(|(i, v)| if v.id == id { Some(i) } else { None });
        if let Some(index) = index {
            return Some(self.children.remove(index));
        }
        //Stupid bullshit design of im crate: not able to implement a fucking IntoIterator for &mut!
        for v in self.children.iter_mut() {
            if let Some(node) = v.remove_node(id) {
                return Some(node);
            }
        }
        None
    }

    /// Inserts a new child node to this node.
    pub fn add_node(&mut self, node: Span) {
        self.children.push(node);
    }

    /// Attempts to add the specified node under the specified parent.
    ///
    /// If the parent could not be found the node is returned.
    /// If the parent was found and the node added None is returned.
    pub fn add_node_with_parent(&mut self, node: Span, parent: u32) -> Option<Span> {
        if self.id == parent {
            self.add_node(node);
            return None;
        }
        let mut node = node;
        for v in self.children.iter_mut() {
            match v.add_node_with_parent(node, parent) {
                Some(v) => node = v,
                None => return None,
            }
        }
        Some(node)
    }

    /// Attempts to relocated the specified node under the new specified parent.
    ///
    /// Returns true if the operation has succeeded.
    pub fn relocate_node(&mut self, id: u32, new_parent: u32) -> bool {
        if let Some(node) = self.remove_node(id) {
            if self.add_node_with_parent(node, new_parent).is_none() {
                return true;
            }
        }
        return false;
    }

    pub async fn write<T: AsyncWrite + AsyncWriteExt + Unpin>(&self, file: &mut T) -> Result<()> {
        let mut queue = VecDeque::new();
        queue.push_back((self.metadata.name.clone(), self));
        while let Some((path, elem)) = queue.pop_front() {
            file.write_all(format!("{} {}\n", path, elem.id).as_bytes())
                .await?;
            for child in &elem.children {
                queue.push_back((format!("{}/{}", path, child.metadata.name), child))
            }
        }
        Ok(())
    }
}

impl Default for Span {
    fn default() -> Self {
        Self::new()
    }
}
