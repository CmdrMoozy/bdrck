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
use crate::http::client::AbstractClient;
use crate::http::recording::{RecordedBody, RecordedRequest, Recording, RecordingEntry};
use crate::http::types::ResponseMetadata;
use reqwest::Client as InnerClient;
use reqwest::{Request, RequestBuilder, Url};
use serde_json;
use std::collections::VecDeque;
use std::sync::Mutex;

/// TestStubClient provides an HTTP-client-like interface for unit testing.
/// Instead of interacting with real servers, it loads a previously recorded
/// HTTP session and verifies application behavior against it.
pub struct TestStubClient {
    inner: InnerClient,
    recordings: Mutex<VecDeque<Recording>>,
}

impl TestStubClient {
    /// Create a new, empty test stub client.
    pub fn new() -> Self {
        TestStubClient {
            inner: InnerClient::new(),
            recordings: Mutex::new(VecDeque::new()),
        }
    }

    /// Push the given recording (the serialized bytes) into this test stub.
    pub fn push_recording(&self, recording: &[u8]) -> Result<&Self> {
        self.recordings
            .lock()
            .unwrap()
            .push_back(serde_json::from_slice(recording)?);
        Ok(self)
    }
}

impl AbstractClient for TestStubClient {
    fn execute(&self, request: Request) -> Result<(ResponseMetadata, Vec<u8>)> {
        // Get the next RecordingEntry out, and pop empty Recordings (if any).

        let entry: RecordingEntry;
        let pop: bool;
        let mut recordings = self.recordings.lock().unwrap();

        {
            let recording = match recordings.front_mut() {
                None => {
                    panic!("Unexpected call to AbstractClient::execute (no more mock recordings)")
                }
                Some(recording) => recording,
            };
            entry = recording.0.pop_front().unwrap();
            pop = recording.0.is_empty();
        }

        if pop {
            recordings.pop_front();
        }

        // Make sure the request matches what we're expecting.
        let assert_req = RecordedRequest::from(&request);
        assert_eq!(
            entry.req, assert_req,
            "HTTP server expected {:#?}, got {:#?}",
            entry.req, assert_req
        );

        Ok((
            entry.res.metadata,
            match entry.res.body {
                RecordedBody::Text(text) => text.into_bytes(),
                RecordedBody::Binary(bytes) => bytes,
            },
        ))
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

impl Drop for TestStubClient {
    fn drop(&mut self) {
        // Technically it is invalid (gives SIGILL) to panic inside drop(), but
        // users of this struct often don't have access to assert this
        // themselves (because HTTP-related structs often take sole ownership of
        // their AbstractClient). So, print out a warning, and panic anyway.
        //
        // Try to sidestep this problem a bit by skipping this assertion if
        // we're already panicking anyway.
        if !::std::thread::panicking() {
            let empty = self.recordings.lock().unwrap().is_empty();
            if !empty {
                println!("Test failure / panic: test ended with mock HTTP client recordings still pending");
            }
            assert!(empty);
        }
    }
}
