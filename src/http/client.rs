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

use error::*;
// For recordings.
#[cfg(debug_assertions)]
use http::recording::{RecordedRequest, RecordedResponse, Recording, RecordingEntry};
use http::types::ResponseMetadata;
use reqwest::Client as InnerClient;
use reqwest::{Request, RequestBuilder, Url};
use std::io::Read;
// For recordings.
#[cfg(debug_assertions)]
use std::path::{Path, PathBuf};
// For recordings.
#[cfg(debug_assertions)]
use std::sync::Mutex;

/// AbstractClient defines the generic interface for an HTTP client.
pub trait AbstractClient {
    /// Execute (send) a previously-constructed HTTP request.
    fn execute(&self, request: Request) -> Result<(ResponseMetadata, Vec<u8>)>;

    /// Returns a builder for an HTTP GET request.
    fn get(&self, url: Url) -> RequestBuilder;
    /// Returns a builder for an HTTP POST request.
    fn post(&self, url: Url) -> RequestBuilder;
    /// Returns a builder for an HTTP PUT request.
    fn put(&self, url: Url) -> RequestBuilder;
    /// Returns a builder for an HTTP PATCH request.
    fn patch(&self, url: Url) -> RequestBuilder;
    /// Returns a builder for an HTTP DELETE request.
    fn delete(&self, url: Url) -> RequestBuilder;
    /// Returns a builder for an HTTP HEAD request.
    fn head(&self, url: Url) -> RequestBuilder;
}

/// Client is the standard, non-testing implementation of AbstractClient. If
/// debug assersions are enabled, then this structure also provides a mechanism
/// for recording an HTTP session.
pub struct Client {
    inner: InnerClient,
    #[cfg(debug_assertions)]
    recording: Option<Mutex<Recording>>,
    #[cfg(debug_assertions)]
    recording_output: Option<PathBuf>,
}

impl Client {
    /// Initialize a new client.
    pub fn new() -> Self {
        Client {
            inner: InnerClient::new(),
            #[cfg(debug_assertions)]
            recording: None,
            #[cfg(debug_assertions)]
            recording_output: None,
        }
    }

    /// Initialize a new client, which will record its HTTP session and write
    /// the result to the given path once it is destructed.
    #[cfg(debug_assertions)]
    pub fn new_with_recording<P: AsRef<Path>>(recording_output: P) -> Self {
        Client {
            inner: InnerClient::new(),
            recording: Some(Mutex::new(Recording::default())),
            recording_output: Some(recording_output.as_ref().to_path_buf()),
        }
    }

    fn execute_impl(&self, request: Request) -> Result<(ResponseMetadata, Vec<u8>)> {
        #[cfg(debug_assertions)]
        let method = request.method().clone();
        #[cfg(debug_assertions)]
        let url = request.url().clone();

        let mut res = self.inner.execute(request)?;
        let metadata = ResponseMetadata::from(&res);
        let mut body: Vec<u8> = Vec::new();
        res.read_to_end(&mut body)?;

        #[cfg(debug_assertions)]
        debug!("{} {} => {}", method, url, metadata.get_status().unwrap());

        Ok((metadata, body))
    }
}

impl AbstractClient for Client {
    #[cfg(not(debug_assertions))]
    fn execute(&self, request: Request) -> Result<(ResponseMetadata, Vec<u8>)> {
        self.execute_impl(request)
    }

    #[cfg(debug_assertions)]
    fn execute(&self, request: Request) -> Result<(ResponseMetadata, Vec<u8>)> {
        let recorded_req = RecordedRequest::from(&request);
        let res = self.execute_impl(request)?;

        if let Some(recording) = self.recording.as_ref() {
            let recorded_res = RecordedResponse::from(&res);
            let mut lock = recording.lock().unwrap();
            lock.0.push_back(RecordingEntry {
                req: recorded_req,
                res: recorded_res,
            });
        }

        Ok(res)
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

#[cfg(debug_assertions)]
impl Drop for Client {
    fn drop(&mut self) {
        if let Some(recording_output) = self.recording_output.as_ref() {
            // We want to write the recording to the given file. Note that we're
            // happy to panic if things go awry here, because this is purely a
            // debugging / testing feature anyway.
            self.recording
                .as_ref()
                .unwrap()
                .lock()
                .unwrap()
                .flush(recording_output)
                .unwrap();
            debug!(
                "Wrote HTTP client recording to: {}",
                recording_output.display()
            );
        }
    }
}
