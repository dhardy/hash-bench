extern crate seahash;
extern crate metrohash;

use std::hash::Hasher;
use metrohash::MetroHash128;
use seahash::SeaHasher;

fn main() {
    let text = "Hello, world!";
    println!("Text: {}", text);
    
    let mut hash = MetroHash128::new();
    hash.write(text.as_bytes());
    println!("MetroHash128: {:?}", hash.finish128());
    
    let mut hash = SeaHasher::new();
    hash.write(text.as_bytes());
    println!("SeaHasher: {}", hash.finish());
}
