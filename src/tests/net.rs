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

use net::*;
use std::net::IpAddr;

macro_rules! ip {
    ($e:expr) => (
        $e.parse::<IpAddr>().unwrap()
    )
}

macro_rules! mac {
    ($e:expr) => (
        $e.parse::<HardwareAddr>().unwrap()
    )
}

macro_rules! net {
    ($e:expr) => (
        $e.parse::<IpNet>().unwrap()
    )
}

#[test]
fn test_increment_ip() {
    assert_eq!(Some(ip!("10.0.0.1")), increment_ip(ip!("10.0.0.0")));
    assert_eq!(Some(ip!("10.10.10.11")), increment_ip(ip!("10.10.10.10")));
    assert_eq!(Some(ip!("10.0.1.0")), increment_ip(ip!("10.0.0.255")));
    assert_eq!(None, increment_ip(ip!("255.255.255.255")));
}

#[test]
fn test_min_max_ip() {
    assert_eq!(
        Some(ip!("10.0.0.1")),
        min_ip(ip!("10.0.0.2"), ip!("10.0.0.1"))
    );
    assert_eq!(
        Some(ip!("10.0.0.2")),
        max_ip(ip!("10.0.0.2"), ip!("10.0.0.1"))
    );
    assert_eq!(None, min_ip(ip!("10.0.0.1"), ip!("::1")));
    assert_eq!(None, max_ip(ip!("10.0.0.1"), ip!("::1")));
}

#[test]
fn test_hardware_addr_string_round_trip() {
    assert_eq!("0c:c4:7a:7f:b6:32", mac!("0c:c4:7a:7f:b6:32").to_string());
    assert_eq!("0c:c4:7a:7f:b6:32", mac!("0c-c4-7a-7f-b6-32").to_string());
    assert_eq!("0c:c4:7a:7f:b6:32", mac!("0cc4.7a7f.b632").to_string());
}

#[test]
fn test_hardware_addr_parse_error() {
    assert!("00:00:00:00:00:00:00".parse::<HardwareAddr>().is_err());
    assert!("00:00:00:00:00".parse::<HardwareAddr>().is_err());
    assert!("00_00_00_00_00_00".parse::<HardwareAddr>().is_err());
    assert!("xx:xx:xx:xx:xx:xx".parse::<HardwareAddr>().is_err());
}

#[test]
fn test_hardware_addr_string_bit_order() {
    let parsed = mac!("12-34-56-78-9A-BC");
    assert_eq!(
        &[0x12_u8, 0x34_u8, 0x56_u8, 0x78_u8, 0x9a_u8, 0xbc_u8],
        parsed.as_bytes()
    );
    assert_eq!("12:34:56:78:9a:bc", parsed.to_string());
}

#[test]
fn test_ip_net_string_round_trip() {
    assert_eq!("10.0.0.0/24", net!("10.0.0.0/24").to_string());
    assert_eq!("10.0.0.0/14", net!("10.0.0.0/14").to_string());
    assert_eq!("10.0.0.0/14", net!("10.0.0.0/fffc0000").to_string());
}

#[test]
fn test_ip_net_bit_order() {
    assert_eq!(
        [
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0x00,
        ],
        net!("10.0.0.0/24").get_mask()
    );
    assert_eq!(
        [
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xfc,
            0x00, 0x00,
        ],
        net!("10.0.0.0/14").get_mask()
    );
    assert_eq!(
        [
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x0a, 0x37,
            0x50, 0x4e,
        ],
        net!("10.0.0.0/0a37504e").get_mask()
    );
    assert_eq!(
        [
            0x4e, 0xd6, 0x2c, 0x4b, 0x1f, 0xb2, 0xbb, 0x41, 0x80, 0x80, 0x1d, 0xe9, 0xcf, 0x77,
            0xd1, 0x6e,
        ],
        net!("::1/4ed62c4b1fb2bb4180801de9cf77d16e").get_mask()
    );
}

#[test]
fn test_ip_net_netmask() {
    assert_eq!(ip!("255.255.0.0"), net!("10.10.0.0/16").netmask());
    assert_eq!(ip!("255.255.255.0"), net!("10.10.0.0/24").netmask());
    assert_eq!(ip!("255.255.240.0"), net!("10.10.0.0/20").netmask());
}

#[test]
fn test_ip_net_broadcast() {
    assert_eq!(ip!("10.10.255.255"), net!("10.10.0.0/16").broadcast());
    assert_eq!(ip!("10.10.10.255"), net!("10.10.10.0/24").broadcast());
    assert_eq!(ip!("172.31.255.255"), net!("172.16.0.0/12").broadcast());
}

#[test]
fn test_ip_net_contains() {
    assert_eq!(
        true,
        net!("10.10.10.0/24").contains(ip!("10.10.10.123"), false)
    );
    assert_eq!(
        false,
        net!("10.10.10.0/24").contains(ip!("10.10.0.123"), false)
    );
    assert_eq!(
        true,
        net!("10.10.10.0/24").contains(ip!("10.10.10.0"), false)
    );
    assert_eq!(
        false,
        net!("10.10.10.0/24").contains(ip!("10.10.10.0"), true)
    );
    assert_eq!(
        true,
        net!("10.10.10.0/24").contains(ip!("10.10.10.255"), false)
    );
    assert_eq!(
        false,
        net!("10.10.10.0/24").contains(ip!("10.10.10.255"), true)
    );
}

#[test]
fn test_ip_net_first_address() {
    assert_eq!(Some(ip!("10.10.10.1")), net!("10.10.10.0/24").first());
    assert_eq!(Some(ip!("10.10.0.1")), net!("10.10.0.0/16").first());
}

#[test]
fn test_ip_net_last_address() {
    assert_eq!(ip!("10.10.10.254"), net!("10.10.10.0/24").last());
    assert_eq!(ip!("10.10.255.254"), net!("10.10.0.0/16").last());
}
