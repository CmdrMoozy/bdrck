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

// This module contains some dumb shim code to make our structures look like sodiumoxide's. Why?
// Because sodiumoxide didn't just use serde derives, it defined some custom serialize /
// deserialize functions which are incompatible with the derives. So we have to do it this way, or
// we break format compatibility with older versions of the library which used sodiumoxide
// directly.

use crate::crypto::digest::SALT_BYTES;
use crate::crypto::key::NONCE_BYTES;
use crate::error::{Error, Result};
use serde::de::{SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::result::Result as StdResult;

pub(crate) trait Compatible {
    fn from_slice(bytes: &[u8]) -> Result<Self>
    where
        Self: Sized;
}

macro_rules! compat_type {
    ( $(#[$meta:meta])* $name:ident($bytes:expr); ) => (
        $(#[$meta])*
        #[derive(Clone, Eq, PartialEq)]
        pub(crate) struct $name(pub(crate) [u8; $bytes]);

        impl Default for $name {
            fn default() -> Self {
                $name([0; $bytes])
            }
        }

        impl fmt::Debug for $name {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                for b in self.0 {
                    write!(f, "{:02x}", b)?;
                }
                Ok(())
            }
        }

        impl Compatible for $name {
            fn from_slice(bytes: &[u8]) -> Result<Self> {
                if bytes.len() != $bytes {
                    return Err(Error::InvalidArgument(format!("invalid {}, expected {} bytes, found {}", stringify!($name), $bytes, bytes.len())));
                }

                let mut x = Self::default();
                x.0.as_mut_slice().copy_from_slice(bytes);
                Ok(x)
            }
        }

        impl Serialize for $name {
            fn serialize<S: Serializer>(&self, serializer: S) -> StdResult<S::Ok, S::Error> {
                serializer.serialize_bytes(&self.0)
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D: Deserializer<'de>>(deserializer: D) -> StdResult<Self, D::Error> {
                struct MyVisitor;

                impl<'de> Visitor<'de> for MyVisitor {
                    type Value = $name;

                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        write!(formatter, "{}", stringify!($name))
                    }

                    fn visit_seq<V: SeqAccess<'de>>(self, mut visitor: V) -> StdResult<Self::Value, V::Error> {
                        let mut res = Self::Value::default();
                        for b in res.0.iter_mut() {
                            if let Some(vb) = visitor.next_element()? {
                                *b = vb;
                            }
                        }
                        Ok(res)
                    }

                    fn visit_bytes<E: serde::de::Error>(self, v: &[u8]) -> StdResult<Self::Value, E> {
                        Self::Value::from_slice(v).map_err(|_| serde::de::Error::invalid_length(v.len(), &self))
                    }
                }

                deserializer.deserialize_bytes(MyVisitor)
            }
        }
    );
}

compat_type! { Salt(SALT_BYTES); }
compat_type! { Nonce(NONCE_BYTES); }
