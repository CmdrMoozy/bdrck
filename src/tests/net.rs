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
