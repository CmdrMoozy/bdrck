# halite-sys

halite-sys provides unsafe Rust bindings for [libsodium](https://doc.libsodium.org/).

This crate tracks upstream's [stable branch](https://github.com/jedisct1/libsodium/tree/stable). In general, upstream appears to publish fixes to this branch regularly, whereas new full release versions aren't released very frequently. In general, tracking this branch seems reasonable based on the [documentation](https://doc.libsodium.org/installation#stable-branch) describing it.

Why not use [libsodium-sys](https://crates.io/crates/libsodium-sys) or [sodiumoxide](https://crates.io/crates/sodiumoxide)? Because those crates are [deprecated](https://github.com/sodiumoxide/sodiumoxide/blob/master/README.md). Further, those crates have [known security vulnerabilities](https://www.cvedetails.com/cve/CVE-2019-25002/).
