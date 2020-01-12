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
use crate::http::types::{HttpData, ResponseMetadata};
use reqwest::Request;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::{HashMap, VecDeque};
use std::fs::File;
use std::io::Write;
use std::path::Path;

/// RecordedRequest represents a recorded HTTP request.
#[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct RecordedRequest {
    /// The HTTP method (verb), as a string.
    pub method: String,
    /// The URL to which the request was sent.
    pub url: String,
    /// The headers sent along with the request (if any).
    pub headers: HashMap<String, Vec<HttpData>>,
    /// The request body (if any).
    pub body: Option<String>,
}

impl<'a> From<&'a Request> for RecordedRequest {
    fn from(req: &'a Request) -> Self {
        let mut headers = HashMap::new();
        for (name, value) in req.headers().iter() {
            let value: HttpData = match value.to_str() {
                Ok(s) => HttpData::Text(s.to_owned()),
                Err(_) => HttpData::Binary(value.as_bytes().to_vec()),
            };
            let entry = headers
                .entry(name.as_str().to_owned())
                .or_insert_with(Vec::new);
            (*entry).push(value);
        }

        RecordedRequest {
            method: req.method().to_string(),
            url: req.url().as_str().to_owned(),
            headers: headers,
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
    pub body: HttpData,
}

impl<'a> From<&'a (ResponseMetadata, Vec<u8>)> for RecordedResponse {
    fn from(res: &'a (ResponseMetadata, Vec<u8>)) -> Self {
        RecordedResponse {
            metadata: res.0.clone(),
            body: HttpData::from(res.1.as_slice()),
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
