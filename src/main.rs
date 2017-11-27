extern crate seahash;
extern crate metrohash;

use std::hash::Hasher;
use metrohash::MetroHash128;
use seahash::SeaHasher;
use highwayhash::HighwayHash;

mod highwayhash;

fn main() {
    let text = "Hello, world!";
    println!("Text: {}", text);
    
    let mut hasher = MetroHash128::new();
    hasher.write(text.as_bytes());
    println!("MetroHash128: {:?}", hasher.finish128());
    
    let mut hasher = SeaHasher::new();
    hasher.write(text.as_bytes());
    println!("SeaHasher: {}", hasher.finish());
    
    let mut hasher = HighwayHash::new_fixed();
    hasher.write(text.as_bytes());
    println!("HighwayHash: {}", hasher.finalize_64());
}
