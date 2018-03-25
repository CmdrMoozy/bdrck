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

use data_encoding::HEXLOWER_PERMISSIVE;
use error::*;
use serde::de::{Deserialize, Deserializer, Unexpected, Visitor};
use serde::ser::{Serialize, Serializer};
use std::fmt;
use std::marker::PhantomData;
use std::str::FromStr;

struct ParseableVisitor<T: FromStr<Err = Error>> {
    phantom: PhantomData<T>,
}

impl<'de, T: FromStr<Err = Error>> Visitor<'de> for ParseableVisitor<T> {
    type Value = T;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a parseable value")
    }

    fn visit_str<E: ::serde::de::Error>(self, v: &str) -> ::std::result::Result<Self::Value, E> {
        match v.parse::<T>() {
            Err(e) => Err(E::invalid_value(
                Unexpected::Str(v),
                &e.to_string().as_str(),
            )),
            Ok(addr) => Ok(addr),
        }
    }
}

impl<T: FromStr<Err = Error>> Default for ParseableVisitor<T> {
    fn default() -> Self {
        ParseableVisitor {
            phantom: PhantomData,
        }
    }
}

/// This structure denotes a hardware MAC address.
#[derive(Clone, Copy, Debug, Hash, Eq, Ord, PartialEq, PartialOrd)]
pub struct HardwareAddr {
    address: [u8; 6],
}

impl HardwareAddr {
    /// Return the 6 bytes which make up the MAC address as a slice.
    pub fn as_bytes(&self) -> &[u8] {
        &self.address
    }
}

impl fmt::Display for HardwareAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:02x}:{:02x}:{:02x}:{:02x}:{:02x}:{:02x}",
            self.address[0],
            self.address[1],
            self.address[2],
            self.address[3],
            self.address[4],
            self.address[5]
        )
    }
}

impl FromStr for HardwareAddr {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        // Common / valid MAC address formats are "xx:xx:xx:xx:xx:xx",
        // "xx-xx-xx-xx-xx-xx", and "xxxx.xxxx.xxxx" where "xx" is a hex byte.
        let normalized = s.replace(":", "");
        let normalized = normalized.replace("-", "");
        let normalized = normalized.replace(".", "");
        let address_vec = HEXLOWER_PERMISSIVE.decode(normalized.as_bytes())?;
        if address_vec.len() != 6 {
            bail!(
                "Invalid MAC address '{}', expected 6 bytes found {}",
                s,
                address_vec.len()
            );
        }

        let mut address = [0_u8; 6];
        for (dst, src) in address.iter_mut().zip(address_vec.into_iter()) {
            *dst = src;
        }

        Ok(HardwareAddr { address: address })
    }
}

impl Serialize for HardwareAddr {
    fn serialize<S: Serializer>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error> {
        serializer.serialize_str(self.to_string().as_str())
    }
}

impl<'de> Deserialize<'de> for HardwareAddr {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> ::std::result::Result<Self, D::Error> {
        deserializer.deserialize_str(ParseableVisitor::<HardwareAddr>::default())
    }
}
