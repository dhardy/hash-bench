// Implementation of K12, based on the reference implementation:
// https://github.com/gvanas/KeccakCodePackage/blob/master/Standalone/k12s-reference/K12.py
//
// Some optimisations copied from https://github.com/RustCrypto/hashes/tree/master/sha3/src
//
// To the extent possible under law, the implementer has waived all copyright
// and related or neighboring rights to the source code in this file.
// http://creativecommons.org/publicdomain/zero/1.0/

use std::cmp::min;

#[macro_use]
mod macros {
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

    macro_rules! REPEAT4 {
        ($e: expr) => ( $e; $e; $e; $e; )
    }

    macro_rules! REPEAT5 {
        ($e: expr) => ( $e; $e; $e; $e; $e; )
    }

    macro_rules! REPEAT6 {
        ($e: expr) => ( $e; $e; $e; $e; $e; $e; )
    }

    macro_rules! REPEAT24 {
        ($e: expr, $s: expr) => (
            REPEAT6!({ $e; $s; });
            REPEAT6!({ $e; $s; });
            REPEAT6!({ $e; $s; });
            REPEAT5!({ $e; $s; });
            $e;
        )
    }

    macro_rules! FOR5 {
        ($v: expr, $s: expr, $e: expr) => {
            $v = 0;
            REPEAT4!({
                $e;
                $v += $s;
            });
            $e;
        }
    }
}

mod lanes {
    pub const RC: [u64; 12] = [
        0x000000008000808b,
        0x800000000000008b,
        0x8000000000008089,
        0x8000000000008003,
        0x8000000000008002,
        0x8000000000000080,
        0x000000000000800a,
        0x800000008000000a,
        0x8000000080008081,
        0x8000000000008080,
        0x0000000080000001,
        0x8000000080008008,
    ];

    // (0..24).map(|t| ((t+1)*(t+2)/2) % 64)
    pub const RHO: [u32; 24] = [
        1, 3, 6, 10, 15, 21,28, 36, 45, 55, 2, 14, 27,
        41, 56, 8, 25, 43, 62, 18, 39, 61, 20, 44
    ];
    pub const PI: [usize; 24] = [
        10, 7, 11, 17, 18, 3, 5, 16, 8, 21, 24, 4, 15, 23,
        19, 13, 12, 2, 20, 14, 22, 9, 6, 1
    ];

    pub fn keccak(lanes: &mut [u64; 25]) {
        let mut c = [0u64; 5];
        let (mut x, mut y): (usize, usize);
        
        for round in 0..12 {
            // θ
            FOR5!(x, 1, {
                c[x] = lanes[x] ^ lanes[x+5] ^ lanes[x+10] ^ lanes[x+15] ^ lanes[x+20];
            });
            
            FOR5!(x, 1, {
                FOR5!(y, 5, {
                    lanes[x + y] ^= c[(x+4)%5] ^ c[(x+1)%5].rotate_left(1);
                });
            });
            
            // ρ and π
            let mut a = lanes[1];
            x = 0;
            REPEAT24!({
                c[0] = lanes[PI[x]];
                lanes[PI[x]] = a.rotate_left(RHO[x]);
            }, {
                a = c[0];
                x += 1;
            });
            
            // χ
            FOR5!(y, 5, {
                FOR5!(x, 1, {
                    c[x] = lanes[x + y];
                });
                FOR5!(x, 1, {
                    lanes[x + y] = c[x] ^((!c[(x+1) % 5]) & c[(x+2)%5]);
                });
            });
            
            // ι
            lanes[0] ^= RC[round];
        }
    }
}

fn read_u64(bytes: &[u8; 8]) -> u64 {
    unsafe{ *(bytes as *const [u8; 8] as *const u64) }.to_le()
}
fn write_u64(val: u64) -> [u8; 8] {
    unsafe{ *(&val.to_le() as *const u64 as *const [u8; 8]) }
}

fn keccak(state: &mut [u8; 200]) {
    let mut lanes = [0u64; 25];
    let mut y;
    for x in 0..5 {
        FOR5!(y, 5, {
            lanes[x + y] = read_u64(array_ref!(state, 8*(x+y), 8));
        });
    }
    lanes::keccak(&mut lanes);
    for x in 0..5 {
        FOR5!(y, 5, {
            let i = 8*(x+y);
            state[i..i+8].copy_from_slice(&write_u64(lanes[x + y]));
        });
    }
}

pub fn k12s<T: AsRef<[u8]>, O: AsMut<[u8]>+Default>(input: T) -> O {
    let input = input.as_ref();
    let mut state = [0u8; 200];
    let max_block_size = 1344 / 8;  // r, also known as rate in bytes
    
    // === Absorb all the input blocks ===
    // We unroll first loop, which allows simple copy
    let mut block_size = min(input.len(), max_block_size);
    state[0..block_size].copy_from_slice(&input[0..block_size]);
    
    let mut offset = block_size;
    while offset < input.len() {
        keccak(&mut state);
        block_size = min(input.len() - offset, max_block_size);
        for i in 0..block_size {
            // TODO: is this sufficiently optimisable or better to convert to u64 first?
            state[i] ^= input[i+offset];
        }
        offset += block_size;
    }
    
    // === Do the padding and switch to the squeezing phase ===
    state[block_size] ^= 0x07;
    state[max_block_size-1] ^= 0x80;
    keccak(&mut state);
    
    // === Squeeze out all the output blocks ===
    let mut output = O::default();
    {
        offset = 0;
        let output_ref = output.as_mut();
        let mut output_len = output_ref.len();
        loop {
            block_size = min(output_len, max_block_size);
            output_ref[offset..(offset+block_size)].copy_from_slice(&state[0..block_size]);
            output_len -= block_size;
            offset += block_size;
            if output_len == 0 {
                break;
            }
            keccak(&mut state);
        }
    }
    output
}
