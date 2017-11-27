// Copyright 2017 Diggory Hardy and original developers:
// https://github.com/google/highwayhash
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0>
// 
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::num::Wrapping as w;

#[allow(non_camel_case_types)]
type w64 = w<u64>;


/// Low-level API, use for implementing streams etc.
pub struct HighwayHash {
    v0: [w64; 4],
    v1: [w64; 4],
    mul0: [w64; 4],
    mul1: [w64; 4],
}

/// Copied from `arrayref` crate
macro_rules! array_ref {
    ($arr:expr, $offset:expr, $len:expr) => {{
        {
            #[inline]
            unsafe fn as_array<T>(slice: &[T]) -> &[T; $len] {
                &*(slice.as_ptr() as *const [_; $len])
            }
            let offset = $offset;
            let slice = & $arr[offset..offset + $len];
            unsafe {
                as_array(slice)
            }
        }
    }}
}

impl HighwayHash {
    /// Calculate the hash of the given data with 64-bit output.
    /// 
    /// Convenience function around `new`, `write` and `finalize_64`.
    pub fn hash_64(key: [u64; 4], data: &[u8]) -> u64 {
        let mut hasher = HighwayHash::new(key);
        hasher.write(data);
        hasher.finalize_64()
    }
    
    /// Calculate the hash of the given data with 128-bit output.
    /// 
    /// Convenience function around `new`, `write` and `finalize_128`.
    pub fn hash_128(key: [u64; 4], data: &[u8]) -> [u64; 2] {
        let mut hasher = HighwayHash::new(key);
        hasher.write(data);
        hasher.finalize_128()
    }
    
    /// Calculate the hash of the given data with 256-bit output.
    /// 
    /// Convenience function around `new`, `write` and `finalize_256`.
    pub fn hash_256(key: [u64; 4], data: &[u8]) -> [u64; 4] {
        let mut hasher = HighwayHash::new(key);
        hasher.write(data);
        hasher.finalize_256()
    }
    
    
    /// Creates a state with the given key
    pub fn new(key: [u64; 4]) -> Self {
        fn swap(x: u64) -> w64 { w(x >> 32 | x << 32) }
        
        let mul0 = [w(0xdbe6d5d5fe4cce2fu64),
                w(0xa4093822299f31d0),
                w(0x13198a2e03707344),
                w(0x243f6a8885a308d3)];
        let mul1 = [w(0x3bd39e10cb0ef593u64),
                w(0xc0acf169b5f18a8c),
                w(0xbe5466cf34e90c6c),
                w(0x452821e638d01377)];
        let v0 = [mul0[0] ^ w(key[0]),
                mul0[1] ^ w(key[1]),
                mul0[2] ^ w(key[2]),
                mul0[3] ^ w(key[3])];
        let v1 = [mul1[0] ^ swap(key[0]),
                mul1[1] ^ swap(key[1]),
                mul1[2] ^ swap(key[2]),
                mul1[3] ^ swap(key[3])];
        
        HighwayHash { v0, v1, mul0, mul1 }
    }
    
    /// Creates a new state with a fixed key
    pub fn new_fixed() -> Self {
        // some hard-coded random numbers
        HighwayHash::new([0x4ae1e91cf3b5737a, 0x4ea5ac492013cced,
                0xb34430a80d547e23, 0xa77ddfe31c89436d])
    }
    
    /// Write the given data
    pub fn write(&mut self, data: &[u8]) {
        let len = data.len();
        let excess = len % 32;
        let end = len - excess;
        let mut i = 0;
        while i < end {
            self.update_packet(array_ref!(data, i, 32));
            i += 32;
        }
        if excess != 0 {
            self.update_remainder(&data[i..i+excess]);
        }
    }
    
    /// Takes a packet of 32 bytes
    pub fn update_packet(&mut self, packet: &[u8; 32]) {
        fn read_u64(bytes: &[u8; 8]) -> w64 {
            w(unsafe{ *(bytes as *const [u8; 8] as *const u64) }.to_le())
        }
        
        let lanes = [read_u64(array_ref!(packet, 0, 8)),
                read_u64(array_ref!(packet, 8, 8)),
                read_u64(array_ref!(packet, 16, 8)),
                read_u64(array_ref!(packet, 24, 8))];
        self.update(lanes);
    }
    
    /// Adds the final 1..31 bytes, do not use if 0 remain
    pub fn update_remainder(&mut self, bytes: &[u8]) {
        fn rotate_32_by(count: usize, lanes: &mut [w64; 4]) {
            for i in 0..4 {
                let half0 = lanes[i].0 as u32;
                let half1 = (lanes[i].0 >> 32) as u32;
                let low = (half0 << count) | (half0 >> (32 - count));
                let high = (half1 << count) | (half1 >> (32 - count));
                lanes[i] = w(low as u64) | (w(high as u64) << 32);
            }
        }
        
        let len = bytes.len();
        let lm4 = len % 4;
        let l1 = len & !3;
        
        let x = ((len as u64) << 32) + len as u64;
        for i in 0..4 {
            self.v0[i] += w(x);
        }
        rotate_32_by(len, &mut self.v1);
        
        // TODO: what is this logic? It's copied exactly from the C code; why?
        // If lm4!=0 and len<16 then some bytes get missed. If lm4==0 and len>=16
        // then some bytes get copied redundantly. Why not just memcpy anyway?
        let mut packet = [0u8; 32];
        packet[0..l1].copy_from_slice(&bytes[0..l1]);
        if len & 16 != 0 {
            for i in 0..4 {
                packet[28 + i] = bytes[len + i - 4];
            }
        } else if lm4 != 0 {
            packet[16 + 0] = bytes[l1];
            packet[16 + 1] = bytes[l1 + (lm4 >> 1)];
            packet[16 + 2] = bytes[len - 1];
        }
        
        self.update_packet(&packet);
    }
    
    fn update(&mut self, lanes: [w64; 4]) {
        macro_rules! zipper_merge_and_add {
            ($v1:expr, $v0:expr, $add1:expr, $add0:expr) => {{
                let (v0, v1) = ($v0.0, $v1.0);
                let x = (((v0 & 0xff000000) | (v1 & 0xff00000000)) >> 24) |
                        (((v0 & 0xff0000000000) | (v1 & 0xff000000000000)) >> 16) |
                        (v0 & 0xff0000) | ((v0 & 0xff00) << 32) |
                        ((v1 & 0xff00000000000000) >> 8) | (v0 << 56);
                let y = (((v1 & 0xff000000) | (v0 & 0xff00000000)) >> 24) |
                        (v1 & 0xff0000) | ((v1 & 0xff0000000000) >> 16) |
                        ((v1 & 0xff00) << 24) | ((v0 & 0xff000000000000) >> 8) |
                        ((v1 & 0xff) << 48) | (v0 & 0xff00000000000000);
                *$add0 += w(x);
                *$add1 += w(y);
            }}
        }
        
        for i in 0..4 {
            self.v1[i] += self.mul0[i] + lanes[i];
            self.mul0[i] ^= (self.v1[i] & w(0xFFFFFFFF)) * (self.v0[i] >> 32);
            self.v0[i] += self.mul1[i];
            self.mul1[i] ^= (self.v0[i] & w(0xFFFFFFFF)) * (self.v1[i] >> 32);
        }
        zipper_merge_and_add!(self.v1[1], self.v1[0], &mut self.v0[1], &mut self.v0[0]);
        zipper_merge_and_add!(self.v1[3], self.v1[2], &mut self.v0[3], &mut self.v0[2]);
        zipper_merge_and_add!(self.v0[1], self.v0[0], &mut self.v1[1], &mut self.v1[0]);
        zipper_merge_and_add!(self.v0[3], self.v0[2], &mut self.v1[3], &mut self.v1[2]);
    }
    
    
    /// Compute the final hash value.
    pub fn finalize_64(mut self) -> u64 {
        self.final_permutes();
        (self.v0[0] + self.v1[0] + self.mul0[0] + self.mul1[0]).0
    }
    
    /// Compute the final hash value.
    pub fn finalize_128(mut self) -> [u64; 2] {
        self.final_permutes();
        let h0 = self.v0[0] + self.mul0[0] + self.v1[2] + self.mul1[2];
        let h1 = self.v0[1] + self.mul0[1] + self.v1[3] + self.mul1[3];
        [h0.0, h1.0]
    }

    /// Compute the final hash value.
    pub fn finalize_256(mut self) -> [u64; 4] {
        fn modular_reduction(a3_unmasked: w64, a2: w64, a1: w64, a0: w64)
                -> (u64, u64)
        {
            let a3 = a3_unmasked & w(0x3FFF_FFFF_FFFF_FFFF);
            let m1 = a1 ^ ((a3 << 1) | (a2 >> 63)) ^ ((a3 << 2) | (a2 >> 62));
            let m0 = a0 ^ (a2 << 1) ^ (a2 << 2);
            (m0.0, m1.0)
        }
        
        self.final_permutes();
        let (h0, h1) = modular_reduction(self.v1[1] + self.mul1[1],
                self.v1[0] + self.mul1[0],
                self.v0[1] + self.mul0[1],
                self.v0[0] + self.mul0[0]);
        let (h2, h3) = modular_reduction(self.v1[3] + self.mul1[3],
                self.v1[2] + self.mul1[2],
                self.v0[3] + self.mul0[3],
                self.v0[2] + self.mul0[2]);
        [h0, h1, h2, h3]
    }
    
    fn final_permutes(&mut self) {
        for _ in 0..4 {
            let v = self.v0;
            let permuted = [(v[2] >> 32) | (v[2] << 32),
                    (v[3] >> 32) | (v[3] << 32),
                    (v[0] >> 32) | (v[0] << 32),
                    (v[1] >> 32) | (v[1] << 32)];
            self.update(permuted);
        }
    }
}
