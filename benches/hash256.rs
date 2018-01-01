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
    kangaroo_twelve(bytes, "", 32)  // 32 * 8 = 256
}
fn k12s(bytes: &[u8]) -> [u8; 32] {
    hash_bench::k12s(bytes)
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

hash256_bytes!(hash256_u64arr_1_k12s, k12s, 1, 100);
hash256_bytes!(hash256_u64arr_4_k12s, k12s, 4, 25);
hash256_bytes!(hash256_u64arr_25_k12s, k12s, 25, 4);

hash256_bytes!(hash256_u64arr_1_sha2, sha512_trunc256, 1, 100);
hash256_bytes!(hash256_u64arr_4_sha2, sha512_trunc256, 4, 25);
hash256_bytes!(hash256_u64arr_25_sha2, sha512_trunc256, 25, 4);

hash256_bytes!(hash256_u64arr_1_sha3, sha3_256, 1, 100);
hash256_bytes!(hash256_u64arr_4_sha3, sha3_256, 4, 25);
hash256_bytes!(hash256_u64arr_25_sha3, sha3_256, 25, 4);
