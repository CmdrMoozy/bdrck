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
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
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

fn increment_ip_bytes(bytes: &mut [u8]) {
    for byte in bytes.iter_mut().rev() {
        match byte.checked_add(1) {
            None => *byte = 0,
            Some(new_byte) => {
                *byte = new_byte;
                return;
            }
        }
    }
}

/// Returns the IP address which immediately follows the given IP address. If
/// the increment overflowed (i.e., the given input IP was already the largest
/// possible IP address), None is returned instead.
pub fn increment_ip(ip: IpAddr) -> Option<IpAddr> {
    match ip {
        IpAddr::V4(ip) => {
            let mut bytes: [u8; 4] = ip.octets();
            increment_ip_bytes(&mut bytes);
            if bytes.iter().fold(true, |acc, byte| acc && *byte == 0) {
                return None;
            }
            Some(Ipv4Addr::from(bytes).into())
        }
        IpAddr::V6(ip) => {
            let mut bytes: [u8; 16] = ip.octets();
            increment_ip_bytes(&mut bytes);
            if bytes.iter().fold(true, |acc, byte| acc && *byte == 0) {
                return None;
            }
            Some(Ipv6Addr::from(bytes).into())
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

/// Apply the given mask to the given IP address bytes. See IpNet::apply_mask
/// for details on behavior.
fn apply_ip_mask_bytes(ip: &mut [u8], mask: &[u8], invert: bool, set: bool) {
    debug_assert!(ip.len() == mask.len());
    for (b, m) in ip.iter_mut().zip(mask.iter()) {
        let m: u8 = match invert {
            false => *m,
            true => !*m,
        };

        if set {
            *b |= m;
        } else {
            *b &= m;
        }
    }
}

/// Apply the given mask to the given IP address. See IpNet::apply_mask for
/// details on behavior.
fn apply_ip_mask(ip: IpAddr, mask: &[u8], invert: bool, set: bool) -> IpAddr {
    debug_assert!(mask.len() == 16);
    let mut bytes: [u8; 16] = match ip {
        IpAddr::V4(ip) => ip.to_ipv6_compatible().octets(),
        IpAddr::V6(ip) => ip.octets(),
    };
    apply_ip_mask_bytes(&mut bytes, mask, invert, set);
    let masked_ip = Ipv6Addr::from(bytes);
    match ip.is_ipv4() {
        false => IpAddr::V6(masked_ip),
        true => {
            let bytes = masked_ip.octets();
            IpAddr::V4(Ipv4Addr::new(bytes[12], bytes[13], bytes[14], bytes[15]))
        }
    }
}

/// An IpNet represents an IP network. Networks are typically identified in CIDR
/// notation, like (for example) "192.0.0.0/24".
#[derive(Clone, Copy, Debug, Hash, Eq, PartialEq)]
pub struct IpNet {
    ip: IpAddr,
    mask: [u8; 16],
}

impl IpNet {
    /// Return the network IP. This address will have been normalized, such that
    /// any non-masked bits will have been turned off during construction.
    pub fn get_ip(&self) -> IpAddr {
        self.ip
    }

    /// Return the network mask, as a byte slice. This slice will always be 16
    /// bytes long, even if this is an IPv4 network.
    pub fn get_mask(&self) -> &[u8] {
        &self.mask
    }

    /// Apply this network's mask to the given IP address, returning the
    /// modified copy.
    ///
    /// The "default" behavior is when invert=false and set=false. In this
    /// case, any bits which are "0" in the mask are turned off in the given
    /// IP address' bytes.
    ///
    /// If invert=true, then each bit in the mask is flipped before it is
    /// applied - in other words, we apply the inverse mask.
    ///
    /// If set=true, then we switch from bitwise AND to bitwise OR, meaning
    /// instead of the behavior described above, any bits which are "1" in the
    /// mask are turned *on* in the IP address, and other bits are left
    /// unchanged.
    pub fn apply_mask(&self, ip: IpAddr, invert: bool, set: bool) -> IpAddr {
        apply_ip_mask(ip, &self.mask, invert, set)
    }

    /// Returns the number of "1" bits in this network's mask. Although the mask
    /// is always 16 bytes long, for IPv4 networks only the last 4 bytes of the
    /// mask are considered.
    pub fn get_one_bits(&self) -> usize {
        self.mask
            .iter()
            .skip(if self.ip.is_ipv4() { 12 } else { 0 })
            .fold(0_u32, |acc, &b| acc + b.count_ones()) as usize
    }

    /// Returns whether or not this network's mask is "canonical" - i.e., if all
    /// of its mask's "1" bits are contiguous (no "0" bits in-between them).
    pub fn is_canonical(&self) -> bool {
        let first_zero_bit = match self.mask.iter().position(|b| *b != 0xff_u8) {
            None => return true,
            Some(idx) => idx,
        };
        if self.mask[first_zero_bit].count_zeros() != self.mask[first_zero_bit].trailing_zeros() {
            return false;
        }
        self.mask
            .iter()
            .skip(first_zero_bit + 1)
            .fold(true, |acc, byte| acc && (*byte == 0x00_u8))
    }

    /// Return the netmask IP address for this network.
    pub fn netmask(&self) -> IpAddr {
        self.apply_mask(self.ip, false, true)
    }

    /// Return the broadcast IP address for this network.
    pub fn broadcast(&self) -> IpAddr {
        self.apply_mask(self.ip, true, true)
    }

    /// Return whether or not the given IP address is contained within this
    /// network.
    pub fn contains(&self, ip: IpAddr) -> bool {
        self.apply_mask(ip, false, false) == self.ip
    }

    /// Return the first IP address which falls within this network.
    pub fn first(&self) -> Option<IpAddr> {
        increment_ip(self.ip)
    }

    /// Return the last non-broadcast IP address which falls within this
    /// network.
    pub fn last(&self) -> IpAddr {
        match self.broadcast() {
            IpAddr::V4(ip) => {
                let mut bytes = ip.octets();
                let idx = bytes.len() - 1;
                bytes[idx] -= 1;
                Ipv4Addr::from(bytes).into()
            }
            IpAddr::V6(ip) => {
                let mut bytes = ip.octets();
                let idx = bytes.len() - 1;
                bytes[idx] -= 1;
                Ipv6Addr::from(bytes).into()
            }
        }
    }
}

impl fmt::Display for IpNet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}/{}",
            self.ip,
            match self.is_canonical() {
                false => HEXLOWER_PERMISSIVE
                    .encode(&self.mask)
                    .chars()
                    .skip(if self.ip.is_ipv4() { 24 } else { 0 })
                    .collect::<String>(),
                true => self.get_one_bits().to_string(),
            }
        )
    }
}

impl FromStr for IpNet {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let (ip, mask): (&str, &str) = s.split_at(match s.find('/') {
            None => bail!("Invalid IP network specifier '{}'", s),
            Some(idx) => idx,
        });
        let ip: IpAddr = ip.parse()?;
        let mask: &str = &mask[1..];

        let mut mask_vec: Vec<u8> = vec![
            0xff_u8;
            match ip.is_ipv4() {
                false => 0,
                true => 12,
            }
        ];

        let mask_is_hex = (ip.is_ipv4() && mask.len() == 8) || (ip.is_ipv6() && mask.len() == 32);
        if mask_is_hex {
            mask_vec.extend(HEXLOWER_PERMISSIVE.decode(mask.as_bytes())?.into_iter());
        } else {
            let ones = u8::from_str_radix(mask, 10)?;
            let mut v = vec![0xff_u8; (ones / 8) as usize];
            let extra_ones = ones % 8;
            if extra_ones > 0 {
                v.push((0xff_u8 >> (8 - extra_ones)) << (8 - extra_ones));
            }
            mask_vec.extend(v.into_iter());
        }

        let mut mask = [0_u8; 16];
        for (dst, src) in mask.iter_mut().zip(mask_vec.into_iter()) {
            *dst = src;
        }

        Ok(IpNet {
            ip: apply_ip_mask(ip, &mask, false, false),
            mask: mask,
        })
    }
}

impl Serialize for IpNet {
    fn serialize<S: Serializer>(&self, serializer: S) -> ::std::result::Result<S::Ok, S::Error> {
        serializer.serialize_str(self.to_string().as_str())
    }
}

impl<'de> Deserialize<'de> for IpNet {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> ::std::result::Result<Self, D::Error> {
        deserializer.deserialize_str(ParseableVisitor::<IpNet>::default())
    }
}
