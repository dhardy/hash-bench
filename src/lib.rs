extern crate seahash;
extern crate metrohash;
// extern crate tiny_keccak;
extern crate keccak_hash;

pub use metrohash::{MetroHash64, MetroHash128};

pub use seahash::SeaHasher;
pub use seahash::State as SeaHash;

pub use highwayhash::HighwayHash;

// pub use tiny_keccak::{Keccak, keccak256};
pub use keccak_hash::{H256, keccak};

pub use k12::KangarooTwelve;

mod highwayhash;
mod k12;
