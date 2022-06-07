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

use crate::http::types::{HeaderMap, HttpData};
use crate::http::util::*;
use reqwest::header;
use std::collections::HashMap;

#[test]
fn test_get_links_no_header() {
    crate::init().unwrap();

    let headers = HeaderMap::new();
    assert_eq!(HashMap::new(), get_links(&headers).unwrap());
}

#[test]
fn test_get_links_no_values() {
    crate::init().unwrap();

    let mut headers = HeaderMap::new();
    headers.insert(header::LINK.as_str().to_owned(), Vec::new());
    assert_eq!(HashMap::new(), get_links(&headers).unwrap());
}

#[test]
fn test_get_links_multiple_values_single_urls() {
    crate::init().unwrap();

    let mut headers = HeaderMap::new();
    headers.insert(
        header::LINK.as_str().to_owned(),
        vec![
            HttpData::Text("<http://www.google.com/>; rel=\"first\"".to_owned()),
            HttpData::Text("<http://www.yahoo.com/>; rel=\"next\"".to_owned()),
        ],
    );

    let mut expected = HashMap::new();
    expected.insert(
        "first".to_owned(),
        vec!["http://www.google.com/".parse().unwrap()],
    );
    expected.insert(
        "next".to_owned(),
        vec!["http://www.yahoo.com/".parse().unwrap()],
    );
    assert_eq!(expected, get_links(&headers).unwrap());
}

#[test]
fn test_get_links_multiple_values_multiple_urls() {
    crate::init().unwrap();

    let mut headers = HeaderMap::new();
    headers.insert(
        header::LINK.as_str().to_owned(),
        vec![
            HttpData::Text(
                "<http://www.yahoo.com/>; rel=\"next\", <http://www.google.com/>; rel=\"first\""
                    .to_owned(),
            ),
            HttpData::Text(
                "<http://doc.rust-lang.org/>; rel=\"prev\", <http://www.crates.io/>; rel=\"last\""
                    .to_owned(),
            ),
        ],
    );

    let mut expected = HashMap::new();
    expected.insert(
        "first".to_owned(),
        vec!["http://www.google.com/".parse().unwrap()],
    );
    expected.insert(
        "next".to_owned(),
        vec!["http://www.yahoo.com/".parse().unwrap()],
    );
    expected.insert(
        "prev".to_owned(),
        vec!["http://doc.rust-lang.org/".parse().unwrap()],
    );
    expected.insert(
        "last".to_owned(),
        vec!["http://www.crates.io/".parse().unwrap()],
    );
    assert_eq!(expected, get_links(&headers).unwrap());
}

#[test]
fn test_get_links_multiple_values_empty() {
    crate::init().unwrap();

    let mut headers = HeaderMap::new();
    headers.insert(
        header::LINK.as_str().to_owned(),
        vec![HttpData::Text("".to_owned()), HttpData::Text("".to_owned())],
    );
    assert_eq!(HashMap::new(), get_links(&headers).unwrap());
}

#[test]
fn test_get_links_single_value_multiple_urls() {
    crate::init().unwrap();

    let mut headers = HeaderMap::new();
    headers.insert(
        header::LINK.as_str().to_owned(),
        vec![HttpData::Text(
            "<http://www.yahoo.com/>; rel=\"next\", <http://www.google.com/>; rel=\"first\""
                .to_owned(),
        )],
    );

    let mut expected = HashMap::new();
    expected.insert(
        "first".to_owned(),
        vec!["http://www.google.com/".parse().unwrap()],
    );
    expected.insert(
        "next".to_owned(),
        vec!["http://www.yahoo.com/".parse().unwrap()],
    );
    assert_eq!(expected, get_links(&headers).unwrap());
}

#[test]
fn test_get_links_single_value_single_url() {
    crate::init().unwrap();

    let mut headers = HeaderMap::new();
    headers.insert(
        header::LINK.as_str().to_owned(),
        vec![HttpData::Text(
            "<http://www.google.com/>; rel=\"next\"".to_owned(),
        )],
    );

    let mut expected = HashMap::new();
    expected.insert(
        "next".to_owned(),
        vec!["http://www.google.com/".parse().unwrap()],
    );
    assert_eq!(expected, get_links(&headers).unwrap());
}

#[test]
fn test_get_links_single_value_empty() {
    crate::init().unwrap();

    let mut headers = HeaderMap::new();
    headers.insert(
        header::LINK.as_str().to_owned(),
        vec![HttpData::Text("".to_owned())],
    );
    assert_eq!(HashMap::new(), get_links(&headers).unwrap());
}
