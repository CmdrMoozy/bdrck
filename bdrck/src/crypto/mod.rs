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

/// key defines structures which represent cryptographic keys, and provides
/// some generic code to implement basic operations like encryption, decryption,
/// and key wrapping.
pub mod key;
/// keystore defines a structure for persisting a "master key" on disk, via key
/// wrapping.
pub mod keystore;
