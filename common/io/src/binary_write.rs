// Copyright 2020 Datafuse Labs.
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

use bytes::BufMut;
use common_exception::Result;

use crate::marshal::Marshal;
use crate::stat_buffer::StatBuffer;

const MAX_VARINT_LEN64: usize = 10;

pub trait BinaryWrite {
    fn write_scalar<V>(&mut self, v: &V) -> Result<()>
    where V: Marshal + StatBuffer;

    fn write_string(&mut self, text: impl AsRef<str>) -> Result<()>;
    fn write_uvarint(&mut self, v: u64) -> Result<()>;

    fn write_opt_scalar<V>(&mut self, v: &Option<V>) -> Result<()>
    where V: Marshal + StatBuffer {
        match v {
            Some(v) => {
                self.write_scalar(&1u8)?;
                self.write_scalar(v)
            }
            None => self.write_scalar(&0u8),
        }
    }
}

impl<T> BinaryWrite for T
where T: std::io::Write
{
    fn write_scalar<V>(&mut self, v: &V) -> Result<()>
    where V: Marshal + StatBuffer {
        let mut buffer = V::buffer();
        v.marshal(buffer.as_mut());
        self.write_all(buffer.as_ref())?;
        Ok(())
    }

    fn write_string(&mut self, text: impl AsRef<str>) -> Result<()> {
        let bytes = text.as_ref().as_bytes();
        self.write_uvarint(bytes.len() as u64)?;
        self.write_all(bytes)?;
        Ok(())
    }

    fn write_uvarint(&mut self, v: u64) -> Result<()> {
        let mut scratch = [0u8; MAX_VARINT_LEN64];
        let ln = put_uvarint(&mut scratch[..], v);
        self.write_all(&scratch[..ln])?;
        Ok(())
    }
}

// Another trait like BinaryWrite
// This is aimed to make BufMut to implement it
pub trait BinaryWriteBuf {
    fn write_scalar<V>(&mut self, v: &V) -> Result<()>
    where V: Marshal + StatBuffer;

    fn write_opt_scalar<V>(&mut self, v: &Option<V>) -> Result<()>
    where V: Marshal + StatBuffer {
        match v {
            Some(v) => {
                self.write_scalar(&1u8)?;
                self.write_scalar(v)
            }
            None => self.write_scalar(&0u8),
        }
    }
    fn write_string(&mut self, text: impl AsRef<str>) -> Result<()>;
    fn write_uvarint(&mut self, v: u64) -> Result<()>;
}

// We must ensure there are enough buffer to write because BytesMut do not implicitly grow the buffer.
impl<T> BinaryWriteBuf for T
where T: BufMut
{
    fn write_scalar<V>(&mut self, v: &V) -> Result<()>
    where V: Marshal + StatBuffer {
        let mut buffer = V::buffer();
        v.marshal(buffer.as_mut());

        self.put_slice(buffer.as_ref());
        Ok(())
    }

    fn write_string(&mut self, text: impl AsRef<str>) -> Result<()> {
        let bytes = text.as_ref().as_bytes();
        self.write_uvarint(bytes.len() as u64)?;
        self.put_slice(bytes);
        Ok(())
    }

    fn write_uvarint(&mut self, v: u64) -> Result<()> {
        let mut scratch = [0u8; MAX_VARINT_LEN64];
        let ln = put_uvarint(&mut scratch[..], v);
        self.put_slice(&scratch[..ln]);
        Ok(())
    }
}

// put_uvarint encodes a uint64 into buf and returns the number of bytes written.
// If the buffer is too small, put_uvarint will panic.
pub fn put_uvarint(mut buffer: impl AsMut<[u8]>, x: u64) -> usize {
    let mut i = 0;
    let mut mx = x;
    let buf = buffer.as_mut();
    while mx >= 0x80 {
        buf[i] = mx as u8 | 0x80;
        mx >>= 7;
        i += 1;
    }
    buf[i] = mx as u8;
    i + 1
}
