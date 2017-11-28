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

macro_rules! hash128_u64 {
    ($fnn:ident, $hash:ident) => {
        #[bench]
        fn $fnn(b: &mut Bencher) {
            let mut x: u64 = random();
            
            b.iter(|| {
                for _ in 0..N64 {
                    x = x.wrapping_add(1);  // unique number each time
                    let mut hash = $hash::new();
                    hash.write_u64(x);
                    black_box(hash.finish128());
                }
            });
            b.bytes = 8 * N64;
        }
    }
}

// TODO: why does one result depend on whether another bench is compiled in?
// Does not depend on whether other bench is *run*. Weird optimisation behaviour?
hash128_u64!(hash128_u64_metro, MetroHash128);
// hash128_u64!(hash128_u64_sea, SeaHasher);


macro_rules! hash128_bytes {
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
                    black_box(hash.finish128());
                }
            });
            b.bytes = 8 * $L * $N;
        }
    }
}

// Same as previous test, except as a byte sequence.
hash128_bytes!(hash128_bytes_1_metro, MetroHash128, 1, 100);
// hash128_bytes!(hash128_bytes_1_sea, SeaHasher, 1, 100);

hash128_bytes!(hash128_bytes_4_metro, MetroHash128, 4, 25);
// hash128_bytes!(hash128_bytes_4_sea, SeaHasher, 4, 25);

hash128_bytes!(hash128_bytes_25_metro, MetroHash128, 25, 4);
// hash128_bytes!(hash128_bytes_25_sea, SeaHasher, 25, 4);
