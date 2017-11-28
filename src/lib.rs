extern crate seahash;
extern crate metrohash;
extern crate tiny_keccak;

pub use metrohash::{MetroHash64, MetroHash128};

pub use seahash::SeaHasher;
pub use seahash::State as SeaHash;

pub use highwayhash::HighwayHash;

pub use tiny_keccak::{Keccak, keccak256};

mod highwayhash;
