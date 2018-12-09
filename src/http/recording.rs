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
use crate::http::types::ResponseMetadata;
use reqwest::Request;
use serde_derive::{Deserialize, Serialize};
use serde_json;
use std::collections::{HashMap, VecDeque};
use std::fs::File;
use std::io::Write;
use std::path::Path;

// TODO: Replace with http::types::HttpData.
/// RecordedBody represents a recorded request or response body. It attempts to
/// decode the body as UTF-8, but failing that represents it as raw bytes.
#[derive(Deserialize, Serialize)]
pub enum RecordedBody {
    /// A body which is valid UTF-8.
    Text(String),
    /// A body which is not UTF-8, and therefore represented as raw bytes.
    Binary(Vec<u8>),
}

impl<'a> From<&'a [u8]> for RecordedBody {
    fn from(bytes: &'a [u8]) -> Self {
        match ::std::str::from_utf8(bytes) {
            Err(_) => RecordedBody::Binary(bytes.to_vec()),
            Ok(text) => RecordedBody::Text(text.to_owned()),
        }
    }
}

/// RecordedRequest represents a recorded HTTP request.
#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RecordedRequest {
    /// The HTTP method (verb), as a string.
    pub method: String,
    /// The URL to which the request was sent.
    pub url: String,
    /// The headers sent along with the request (if any).
    pub headers: HashMap<String, Vec<u8>>,
    /// The request body (if any).
    pub body: Option<String>,
}

impl<'a> From<&'a Request> for RecordedRequest {
    fn from(req: &'a Request) -> Self {
        RecordedRequest {
            method: req.method().to_string(),
            url: req.url().as_str().to_owned(),
            headers: req
                .headers()
                .iter()
                .map(|(n, v)| (n.as_str().to_owned(), v.as_bytes().to_vec()))
                .collect(),
            body: req.body().map(|b| format!("{:?}", b)),
        }
    }
}

/// RecordedResponse represents a recorded HTTP response.
#[derive(Deserialize, Serialize)]
pub struct RecordedResponse {
    /// The metadata about the response (e.g. status code, etc.).
    pub metadata: ResponseMetadata,
    /// The response body.
    pub body: RecordedBody,
}

impl<'a> From<&'a (ResponseMetadata, Vec<u8>)> for RecordedResponse {
    fn from(res: &'a (ResponseMetadata, Vec<u8>)) -> Self {
        RecordedResponse {
            metadata: res.0.clone(),
            body: RecordedBody::from(res.1.as_slice()),
        }
    }
}

/// RecordingEntry represents a single entry in a recorded HTTP log, including a
/// request and its matching response.
#[derive(Deserialize, Serialize)]
pub struct RecordingEntry {
    /// The request.
    pub req: RecordedRequest,
    /// The matching response
    pub res: RecordedResponse,
}

/// A Recording is a series of RecordingEntry objects, representing an entire
/// HTTP session.
#[derive(Deserialize, Serialize)]
pub struct Recording(pub VecDeque<RecordingEntry>);

impl Recording {
    /// flush serializes the entire Recording, and writes it out to the given
    /// file on disk (e.g. so it can be loaded and replayed later).
    pub fn flush<P: AsRef<Path>>(&self, output: P) -> Result<()> {
        let mut f = File::create(output)?;
        serde_json::to_writer_pretty(&mut f, self)?;
        f.flush()?;
        Ok(())
    }
}

impl Default for Recording {
    fn default() -> Self {
        Recording(VecDeque::new())
    }
}
