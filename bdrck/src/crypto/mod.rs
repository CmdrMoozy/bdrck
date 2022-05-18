// Copyright 2015 Axel Rasmussen
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

/// digest defines an API for computing cryptographically secure digests of data.
pub mod digest;
/// key defines structures which represent cryptographic keys, and provides some generic code to
/// implement basic operations like encryption and decryption.
pub mod key;
/// keystore defines a structure for persisting a "master key" on disk, via key wrapping.
pub mod keystore;
/// secret defines a structure for "safely" storing "secret" data in memory. Think things like keys,
/// plaintext, etc.
pub mod secret;
mod util;
/// wrap defines utilities for "wrapping" a key with another key. This is useful, for instance, to
/// have a single "master key", which is then encrypted with potentially many other keys. Then, any
/// of the other keys can be used to "unwrap" the "real" master key.
pub mod wrap;
