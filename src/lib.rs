extern crate seahash;
extern crate metrohash;

pub use metrohash::{MetroHash64, MetroHash128};

pub use seahash::SeaHasher;
pub use seahash::State as SeaHash;

pub use highwayhash::HighwayHash;

mod highwayhash;
