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

use crate::error::*;
use crate::http::client::*;
use crate::http::types::{HeaderMap, ResponseMetadata};
use reqwest::Client as InnerClient;
use reqwest::{Method, Request, RequestBuilder, Url};
use std::cell::RefCell;
use std::collections::VecDeque;
use std::time::Duration;

struct RetriesTestClient {
    inner: InnerClient,
    requests: RefCell<VecDeque<Request>>,
    sleeps: RefCell<Vec<Duration>>,
}

impl RetriesTestClient {
    pub(crate) fn new() -> Self {
        RetriesTestClient {
            inner: InnerClient::new(),
            requests: RefCell::new(VecDeque::new()),
            sleeps: RefCell::new(Vec::new()),
        }
    }
}

impl AbstractClient for RetriesTestClient {
    fn execute(&self, request: Request) -> Result<(ResponseMetadata, Vec<u8>)> {
        self.requests.borrow_mut().push_back(request);
        Ok((
            ResponseMetadata {
                status: 503,
                headers: HeaderMap::new(),
            },
            Vec::new(),
        ))
    }

    fn sleep(&self, _: fn(Duration), duration: Duration) {
        self.sleeps.borrow_mut().push(duration);
    }

    fn get(&self, url: Url) -> RequestBuilder {
        self.inner.get(url)
    }
    fn post(&self, url: Url) -> RequestBuilder {
        self.inner.post(url)
    }
    fn put(&self, url: Url) -> RequestBuilder {
        self.inner.put(url)
    }
    fn patch(&self, url: Url) -> RequestBuilder {
        self.inner.patch(url)
    }
    fn delete(&self, url: Url) -> RequestBuilder {
        self.inner.delete(url)
    }
    fn head(&self, url: Url) -> RequestBuilder {
        self.inner.head(url)
    }
}

#[test]
fn test_execute_with_retries_too_many() {
    let client = RetriesTestClient::new();
    assert!(client
        .execute_with_retries(
            59,
            false,
            Method::GET,
            "http://www.google.com/".parse().unwrap(),
            None,
            None
        )
        .is_err());
    assert!(client.requests.borrow().is_empty());
    assert!(client.sleeps.borrow().is_empty());
}

#[test]
fn test_execute_with_retries_single() {
    let client = RetriesTestClient::new();
    assert!(client
        .execute_with_retries(
            0,
            false,
            Method::GET,
            "http://www.google.com/".parse().unwrap(),
            None,
            None
        )
        .is_err());
    assert_eq!(1, client.requests.borrow().len());
    assert!(client.sleeps.borrow().is_empty());
}

#[test]
fn test_execute_with_retries_many() {
    let client = RetriesTestClient::new();
    assert!(client
        .execute_with_retries(
            5,
            false,
            Method::GET,
            "http://www.google.com/".parse().unwrap(),
            None,
            None
        )
        .is_err());
    // We should have sent the request once, plus 5 retries.
    assert_eq!(6, client.requests.borrow().len());
    // This means we should have slept 5 times, once before each retry.
    assert!(vec![
        Duration::from_millis(100),
        Duration::from_millis(200),
        Duration::from_millis(400),
        Duration::from_millis(800),
        Duration::from_millis(1600),
    ]
    .iter()
    .eq(client.sleeps.borrow().iter()),);
}

#[test]
fn test_trait_object_works() {
    let client: Box<dyn AbstractClient> = Box::new(RetriesTestClient::new());
    assert!(client
        .execute_with_retries(
            0,
            false,
            Method::GET,
            "http://www.google.com/".parse().unwrap(),
            None,
            None
        )
        .is_err());
}
