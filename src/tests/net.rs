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

#[test]
fn test_increment_ip() {
    assert_eq!(
        Some("10.0.0.1".parse().unwrap()),
        increment_ip(&"10.0.0.0".parse().unwrap())
    );
    assert_eq!(
        Some("10.10.10.11".parse().unwrap()),
        increment_ip(&"10.10.10.10".parse().unwrap())
    );
    assert_eq!(
        Some("10.0.1.0".parse().unwrap()),
        increment_ip(&"10.0.0.255".parse().unwrap())
    );
    assert_eq!(None, increment_ip(&"255.255.255.255".parse().unwrap()));
}

#[test]
fn test_hardware_addr_string_round_trip() {
    assert_eq!(
        "0c:c4:7a:7f:b6:32",
        "0c:c4:7a:7f:b6:32"
            .parse::<HardwareAddr>()
            .unwrap()
            .to_string()
    );
    assert_eq!(
        "0c:c4:7a:7f:b6:32",
        "0c-c4-7a-7f-b6-32"
            .parse::<HardwareAddr>()
            .unwrap()
            .to_string()
    );
    assert_eq!(
        "0c:c4:7a:7f:b6:32",
        "0cc4.7a7f.b632"
            .parse::<HardwareAddr>()
            .unwrap()
            .to_string()
    );
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
    let parsed: HardwareAddr = "12-34-56-78-9A-BC".parse().unwrap();
    assert_eq!(
        &[0x12_u8, 0x34_u8, 0x56_u8, 0x78_u8, 0x9a_u8, 0xbc_u8],
        parsed.as_bytes()
    );
    assert_eq!("12:34:56:78:9a:bc", parsed.to_string());
}

#[test]
fn test_ip_net_string_round_trip() {
    assert_eq!(
        "10.0.0.0/24",
        "10.0.0.0/24".parse::<IpNet>().unwrap().to_string()
    );
    assert_eq!(
        "10.0.0.0/14",
        "10.0.0.0/14".parse::<IpNet>().unwrap().to_string()
    );
    assert_eq!(
        "10.0.0.0/14",
        "10.0.0.0/fffc0000".parse::<IpNet>().unwrap().to_string()
    );
}

#[test]
fn test_ip_net_bit_order() {
    assert_eq!(
        [
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
            0xff, 0x00,
        ],
        "10.0.0.0/24".parse::<IpNet>().unwrap().get_mask()
    );
    assert_eq!(
        [
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xfc,
            0x00, 0x00,
        ],
        "10.0.0.0/14".parse::<IpNet>().unwrap().get_mask()
    );
    assert_eq!(
        [
            0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x0a, 0x37,
            0x50, 0x4e,
        ],
        "10.0.0.0/0a37504e".parse::<IpNet>().unwrap().get_mask()
    );
    assert_eq!(
        [
            0x4e, 0xd6, 0x2c, 0x4b, 0x1f, 0xb2, 0xbb, 0x41, 0x80, 0x80, 0x1d, 0xe9, 0xcf, 0x77,
            0xd1, 0x6e,
        ],
        "::1/4ed62c4b1fb2bb4180801de9cf77d16e"
            .parse::<IpNet>()
            .unwrap()
            .get_mask()
    );
}

#[test]
fn test_ip_net_contains() {
    assert_eq!(
        true,
        "10.10.10.0/24"
            .parse::<IpNet>()
            .unwrap()
            .contains(&"10.10.10.123".parse().unwrap())
    );
    assert_eq!(
        false,
        "10.10.10.0/24"
            .parse::<IpNet>()
            .unwrap()
            .contains(&"10.10.0.123".parse().unwrap())
    );
}

#[test]
fn test_ip_net_first_address() {
    assert_eq!(
        Some("10.10.10.1".parse().unwrap()),
        "10.10.10.0/24".parse::<IpNet>().unwrap().first()
    );
    assert_eq!(
        Some("10.10.0.1".parse().unwrap()),
        "10.10.0.0/16".parse::<IpNet>().unwrap().first()
    );
}
