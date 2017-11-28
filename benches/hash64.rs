#![feature(test)]

// Benchmarks for 64-bit output

extern crate hash_bench;
extern crate rand;
extern crate test;

use std::hash::Hasher;
use std::slice::from_raw_parts;
use test::{black_box, Bencher};
use rand::random;

use hash_bench::*;

const N64: u64 = 100;

macro_rules! hash64_u64 {
    ($fnn:ident, $hash:ident) => {
        #[bench]
        fn $fnn(b: &mut Bencher) {
            let mut x: u64 = random();
            
            b.iter(|| {
                for _ in 0..N64 {
                    x = x.wrapping_add(1);  // unique number each time
                    let mut hash = $hash::new();
                    hash.write_u64(x);
                    black_box(hash.finish());
                }
            });
            b.bytes = 8 * N64;
        }
    }
}

// TODO: why does one result depend on whether another bench is compiled in?
// Does not depend on whether other bench is *run*. Weird optimisation behaviour?
hash64_u64!(hash64_u64_metro, MetroHash64);
hash64_u64!(hash64_u64_sea, SeaHasher);
// hash64_u64!(hash64_u64_highway, HighwayHash);


macro_rules! hash64_bytes {
    // hash [u64; L] as a byte sequence N times
    ($fnn:ident, $hash:ident, $L:expr, $N:expr) => {
        #[bench]
        fn $fnn(b: &mut Bencher) {
            let mut x: [u64; $L] = random();
            
            b.iter(|| {
                for _ in 0..$N {
                    x[0] = x[0].wrapping_add(1);  // unique number each time
                    let p = &x[0] as *const u64 as *const u8;
                    let slice = unsafe { from_raw_parts(p, x.len() * 8) };
                    let mut hash = $hash::new();
                    hash.write(slice);
                    black_box(hash.finish());
                }
            });
            b.bytes = 8 * $L * $N;
        }
    }
}

// Same as previous test, except as a byte sequence.
// Observation: SeaHash is much slower!
hash64_bytes!(hash64_bytes_1_metro, MetroHash64, 1, 100);
hash64_bytes!(hash64_bytes_1_sea, SeaHasher, 1, 100);
hash64_bytes!(hash64_bytes_1_highway, HighwayHash, 1, 100);

hash64_bytes!(hash64_bytes_4_metro, MetroHash64, 4, 25);
hash64_bytes!(hash64_bytes_4_sea, SeaHasher, 4, 25);
hash64_bytes!(hash64_bytes_4_highway, HighwayHash, 4, 25);

hash64_bytes!(hash64_bytes_25_metro, MetroHash64, 25, 4);
hash64_bytes!(hash64_bytes_25_sea, SeaHasher, 25, 4);
hash64_bytes!(hash64_bytes_25_highway, HighwayHash, 25, 4);

// SeaHash allows usage via a different interface, more optimal?
macro_rules! hash64_buf_sea {
    ($fnn:ident, $L:expr, $N: expr) => {
        #[bench]
        fn $fnn(b: &mut Bencher) {
            let mut x: [u64; $L] = random();
            
            b.iter(|| {
                for _ in 0..$N {
                    x[0] = x[0].wrapping_add(1);  // unique number each time
                    let p = &x[0] as *const u64 as *const u8;
                    let slice = unsafe { from_raw_parts(p, x.len() * 8) };
                    let hash = SeaHash::hash(slice, (0x16f11fe89b0d677c, 0xb480a793d8e6c86c, 0x6fe2e5aaf078ebc9, 0x14f994a4c5259381));
                    black_box(hash.finalize());
                }
            });
            b.bytes = 8 * $L * $N;
        }
    }
}

hash64_buf_sea!(hash64_buf_1_sea, 1, 100);
hash64_buf_sea!(hash64_buf_4_sea, 4, 25);
hash64_buf_sea!(hash64_buf_25_sea, 25, 4);
