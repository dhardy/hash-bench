extern crate hash_bench;

use hash_bench::*;
use std::hash::Hasher;

fn main() {
    let text = "Hello, world!";
    println!("Text: {}", text);
    
    let mut hasher = MetroHash128::new();
    hasher.write(text.as_bytes());
    println!("MetroHash128: {:?}", hasher.finish128());
    
    let mut hasher = SeaHasher::new();
    hasher.write(text.as_bytes());
    println!("SeaHasher: {}", hasher.finish());
    
    let mut hasher = HighwayHash::new();
    hasher.write(text.as_bytes());
    println!("HighwayHash: {}", hasher.finalize_64());
    
    let result = kangaroo_twelve(text, "", 64);
    let result = unsafe{ *(&result[0] as *const u8 as *const u64) }.to_le();
    println!("kangaroo_twelve: {}", result);
}
