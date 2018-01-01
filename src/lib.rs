extern crate seahash;
extern crate metrohash;
// extern crate tiny_keccak;
extern crate keccak_hash;
extern crate sha2;
extern crate sha3;
extern crate generic_array;

pub use metrohash::{MetroHash64, MetroHash128};

pub use seahash::SeaHasher;
pub use seahash::State as SeaHash;

pub use highwayhash::HighwayHash;

// pub use tiny_keccak::{Keccak, keccak256};
pub use keccak_hash::{H256, keccak};

pub use k12::kangaroo_twelve;
pub use k12_simplified::k12s;

pub use sha2::{Digest};
pub use generic_array::{GenericArray, typenum};

mod highwayhash;
mod k12;
mod k12_simplified;

pub fn sha512_trunc256(input: &[u8]) -> GenericArray<u8, typenum::U32> {
    let mut hasher = sha2::Sha512Trunc256::default();
    hasher.input(input);
    hasher.result()
}

pub fn sha3_256(input: &[u8]) -> GenericArray<u8, typenum::U32> {
    let mut hasher = sha3::Sha3_256::default();
    hasher.input(input);
    hasher.result()
}
