// Copyright (c) 2022-2026 Alex Chi Z
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![allow(unused_variables)] // TODO(you): remove this lint after implementing this mod
#![allow(dead_code)] // TODO(you): remove this lint after implementing this mod

use std::sync::Arc;

use bytes::Buf;

use crate::key::{KeySlice, KeyVec};

use super::Block;

/// Iterates on a block.
pub struct BlockIterator {
    /// The internal `Block`, wrapped by an `Arc`
    block: Arc<Block>,
    /// The current key, empty represents the iterator is invalid
    key: KeyVec,
    /// the current value range in the block.data, corresponds to the current key
    value_range: (usize, usize),
    /// Current index of the key-value pair, should be in range of [0, num_of_elements)
    idx: usize,
    /// The first key in the block
    first_key: KeyVec,
}

impl BlockIterator {
    fn new(block: Arc<Block>) -> Self {
        Self {
            block,
            key: KeyVec::new(),
            value_range: (0, 0),
            idx: 0,
            first_key: KeyVec::new(),
        }
    }

    /// Creates a block iterator and seek to the first entry.
    pub fn create_and_seek_to_first(block: Arc<Block>) -> Self {
        let mut iter = Self::new(block);
        iter.seek_to_first();
        iter
    }

    /// Creates a block iterator and seek to the first key that >= `key`.
    pub fn create_and_seek_to_key(block: Arc<Block>, key: KeySlice) -> Self {
        let mut iter = Self::new(block);
        iter.seek_to_key(key);
        iter
    }

    /// Returns the key of the current entry.
    pub fn key(&self) -> KeySlice<'_> {
        self.key.as_key_slice()
    }

    /// Returns the value of the current entry.
    pub fn value(&self) -> &[u8] {
        &self.block.data[self.value_range.0..self.value_range.1]
    }

    /// Returns true if the iterator is valid.
    /// Note: You may want to make use of `key`
    pub fn is_valid(&self) -> bool {
        !self.key.is_empty()
    }

    /// Seeks to the first key in the block.
    pub fn seek_to_first(&mut self) {
        if self.block.offsets.is_empty() {
            self.key = KeyVec::new();
            self.value_range = (0, 0);
            self.idx = 0;
            self.first_key = KeyVec::new();
            return;
        }
        self.idx = 0;
        self.parse_current_key();
        self.first_key = self.key.clone();
    }

    /// Move to the next key in the block.
    pub fn next(&mut self) {
        if !self.is_valid() {
            return;
        }
        self.idx += 1;

        if self.idx >= self.block.offsets.len() {
            self.key = KeyVec::new();
            self.value_range = (0, 0);
            return;
        }
        self.parse_current_key();
    }

    /// Seek to the first key that >= `key`.
    /// Note: You should assume the key-value pairs in the block are sorted when being added by
    /// callers.
    pub fn seek_to_key(&mut self, key: KeySlice) {
        if self.block.offsets.is_empty() {
            self.key = KeyVec::new();
            self.value_range = (0, 0);
            self.idx = 0;
            self.first_key = KeyVec::new();
            return;
        }
        self.seek_to_first();
        if self.key() >= key {
            return;
        }
        while self.is_valid() && self.key() < key {
            self.next();
        }
    }

    fn parse_current_key(&mut self) {
        let offset = usize::from(self.block.offsets[self.idx]);
        let data = &self.block.data[offset..];

        let mut buf = data;

        let overlap = usize::from(buf.get_u16());
        let suffix_len = usize::from(buf.get_u16());

        assert!(
            overlap <= self.first_key.len(),
            "invalid block: overlap {} > first key length {}",
            overlap,
            self.first_key.len()
        );

        let suffix_start = 4;
        let suffix_end = suffix_start + suffix_len;

        assert!(
            suffix_end <= data.len(),
            "invalid block: key suffix exceeds block data"
        );

        let suffix = &data[suffix_start..suffix_end];

        let value_len_offset = suffix_end;

        assert!(
            value_len_offset + 2 <= data.len(),
            "invalid block: missing value length"
        );

        let mut value_len_buf = &data[value_len_offset..];
        let value_len = usize::from(value_len_buf.get_u16());

        let value_start = value_len_offset + 2;
        let value_end = value_start + value_len;

        assert!(
            value_end <= data.len(),
            "invalid block: value exceeds block data"
        );

        let mut key = Vec::with_capacity(overlap + suffix_len);

        key.extend_from_slice(&self.first_key.as_key_slice().raw_ref()[..overlap]);
        key.extend_from_slice(suffix);

        self.key = KeyVec::from_vec(key);

        self.value_range = (offset + value_start, offset + value_end);
    }
}
