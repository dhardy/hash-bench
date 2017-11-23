# Benchmarking some hash functions

Note: as with all micro benchmarks, take the results with a healthy pinch of salt.
In particular, I've noticed that performance depends quite significantly on which
benchmarks are enabled at compile time; not sure why!

Tested:

*   `metrohash::MetroHash64` — fairly fast
*   `metrohash::MetroHash128` — slightly slower than 64 bit variant, but not a big difference
*   `seahash::hash` — similar to `MetroHash64`; maybe a little faster
*   `seahash::SeaHasher` — much slower on buffers (though faster on `u64`)
