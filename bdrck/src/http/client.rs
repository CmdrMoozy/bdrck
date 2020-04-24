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
// For recordings.
#[cfg(debug_assertions)]
use crate::http::recording::{RecordedRequest, RecordedResponse, Recording, RecordingEntry};
use crate::http::types::ResponseMetadata;
use futures::executor::block_on;
use log::{debug, info};
use rand::Rng;
use reqwest::header::HeaderMap;
use reqwest::Client as InnerClient;
use reqwest::{Method, Request, RequestBuilder, Url};
// For recordings.
#[cfg(debug_assertions)]
use std::path::{Path, PathBuf};
// For recordings.
#[cfg(debug_assertions)]
use std::sync::Mutex;
use std::time::Duration;

/// AbstractClient defines the generic interface for an HTTP client.
pub trait AbstractClient {
    /// Execute (send) a previously-constructed HTTP request.
    fn execute(&self, request: Request) -> Result<(ResponseMetadata, Vec<u8>)>;

    /// This function calls the given custom sleep function with the given
    /// Duration. This can be overridden by a trait implementor to add extra
    /// logic, if needed.
    fn sleep(&self, sleep: fn(Duration), duration: Duration) {
        sleep(duration)
    }

    /// Execute (send) a previously-constructed HTTP request. In the case of a
    /// retryable failure (a 5xx error), we'll retry up to max_retries with
    /// exponential backoff between each attempt.
    ///
    /// Unfortunately, to do this we need to be able to create copies of the
    /// request, meaning in particular the Body needs to be copyable. So, this
    /// function can only support Vec<u8>-based request Bodies.
    ///
    /// This function returns the response metadata (e.g. response code) and
    /// body, as well as the number of retries required (i.e., retries + 1
    /// HTTP requests were sent by this function).
    fn execute_with_retries(
        &self,
        max_retries: usize,
        add_jitter: bool,
        method: Method,
        url: Url,
        headers: Option<&HeaderMap>,
        body: Option<&[u8]>,
    ) -> Result<(ResponseMetadata, Vec<u8>)> {
        self.execute_with_retries_custom_sleep(
            std::thread::sleep,
            max_retries,
            add_jitter,
            method,
            url,
            headers,
            body,
        )
    }

    /// This is the same as execute_with_retries, but you can specify a custom
    /// sleep function (as opposed to std::thread::sleep).
    fn execute_with_retries_custom_sleep(
        &self,
        sleep: fn(Duration),
        max_retries: usize,
        add_jitter: bool,
        method: Method,
        url: Url,
        headers: Option<&HeaderMap>,
        body: Option<&[u8]>,
    ) -> Result<(ResponseMetadata, Vec<u8>)> {
        // Below we calculate 2^retry * 100 + 10 as a maximum, so the largest
        // retry value we can store in a u64 is 57 (so max_retries must
        // be <= 58, so retry will be in the range [0, 57)).
        if max_retries > 58 {
            return Err(Error::InvalidArgument(format!("max_retries must be <= 58")));
        }

        let mut rng = rand::thread_rng();
        for retry in 0..max_retries + 1 {
            let mut request = Request::new(method.clone(), url.clone());
            if let Some(headers) = headers {
                (*request.headers_mut()) = headers.clone();
            }
            if let Some(body) = body {
                (*request.body_mut()) = Some(body.to_vec().into());
            }

            if retry > 0 {
                let jitter: u64 = match add_jitter {
                    false => 0,
                    true => rng.gen_range(0, 10),
                };
                let wait: u64 = (1_u64 << retry - 1) * 100 + jitter;
                info!("Sleep for {}ms before retrying {} {}", wait, method, url);
                self.sleep(sleep, Duration::from_millis(wait));
            }

            let (res_metadata, res_body) = self.execute(request)?;
            let status = res_metadata.get_status()?;

            if status.is_server_error() {
                info!("{} {} returned {}, retrying...", method, url, status);
            } else {
                return Ok((res_metadata, res_body));
            }
        }

        Err(Error::HttpRetry(format!(
            "failed to get a success response after {} retries.",
            max_retries
        )))
    }

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

        let res = block_on(self.inner.execute(request))?;
        let metadata = ResponseMetadata::from(&res);
        let body: Vec<u8> = block_on(res.bytes())?.into_iter().collect();

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
