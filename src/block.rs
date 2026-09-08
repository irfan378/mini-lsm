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

mod builder;
mod iterator;

pub use builder::BlockBuilder;
use bytes::{BufMut, Bytes};
pub use iterator::BlockIterator;

pub(crate) const SIZEOF_U16: usize = std::mem::size_of::<u16>();
/// A block is the smallest unit of read and caching in LSM tree. It is a collection of sorted key-value pairs.
pub struct Block {
    pub(crate) data: Vec<u8>,
    pub(crate) offsets: Vec<u16>,
}

impl Block {
    /// Encode the internal data to the data layout illustrated in the course
    /// Note: You may want to recheck if any of the expected field is missing from your output
    pub fn encode(&self) -> Bytes {
        let mut buf = self.data.clone();

        for offset in &self.offsets {
            buf.put_u16(*offset);
        }

        buf.put_u16(self.offsets.len() as u16);

        buf.into()
    }

    /// Decode from the data layout, transform the input `data` to a single `Block`
    pub fn decode(data: &[u8]) -> Self {
        assert!(data.len() >= SIZEOF_U16);
        let num_of_elements =
            u16::from_be_bytes([data[data.len() - 2], data[data.len() - 1]]) as usize;

        assert!(num_of_elements > 0);
        let offsets_size = num_of_elements * SIZEOF_U16;
        let footer_size = offsets_size + SIZEOF_U16;
        assert!(footer_size <= data.len());

        let data_end = data.len() - footer_size;

        let offsets_raw = &data[data_end..data.len() - SIZEOF_U16];

        let mut offsets = Vec::with_capacity(num_of_elements);

        for chunk in offsets_raw.chunks_exact(SIZEOF_U16) {
            let offset = u16::from_be_bytes([chunk[0], chunk[1]]);

            offsets.push(offset);
        }

        let block_data = data[..data_end].to_vec();
        Block {
            data: block_data,
            offsets,
        }
    }
}
