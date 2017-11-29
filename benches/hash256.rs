#![feature(test)]

// Benchmarks for 64-bit output

extern crate hash_bench;
extern crate rand;
extern crate test;

use std::slice::from_raw_parts;
use test::{black_box, Bencher};
use rand::random;

use hash_bench::*;

fn k12(bytes: &[u8]) -> Vec<u8> {
    KangarooTwelve(bytes, "", 256)
}

macro_rules! hash256_bytes {
    // hash [u64; L] as a byte sequence N times
    ($fnn:ident, $hash:expr, $L:expr, $N:expr) => {
        #[bench]
        fn $fnn(b: &mut Bencher) {
            let mut x: [u64; $L] = random();
            
            b.iter(|| {
                for _ in 0..$N {
                    x[0] = x[0].wrapping_add(1);  // unique number each time
                    let p = &x[0] as *const u64 as *const u8;
                    let slice = unsafe { from_raw_parts(p, x.len() * 8) };
                    black_box($hash(slice));
                }
            });
            b.bytes = 8 * $L * $N;
        }
    }
}

hash256_bytes!(hash256_u64arr_1_keccak, keccak, 1, 100);
hash256_bytes!(hash256_u64arr_4_keccak, keccak, 4, 25);
hash256_bytes!(hash256_u64arr_25_keccak, keccak, 25, 4);

hash256_bytes!(hash256_u64arr_1_k12, k12, 1, 100);
hash256_bytes!(hash256_u64arr_4_k12, k12, 4, 25);
hash256_bytes!(hash256_u64arr_25_k12, k12, 25, 4);
