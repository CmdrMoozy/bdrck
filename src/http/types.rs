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
use failure::format_err;
use reqwest::header::HeaderValue;
use reqwest::{Response, StatusCode};
use serde_derive::{Deserialize, Serialize};
use std::collections::HashMap;

/// HTTP data, which is either valid UTF-8 or is treated as binary.
///
/// The HTTP spec in many places allows arbitrary binary data (e.g. in header
/// values, or the request / response body), but it is very common for these to
/// be limited to UTF-8 in practice (e.g. JSON). So, we want to represent the
/// data as a String as often as possible, but we also need to be able to deal
/// with the binary case.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub enum HttpData {
    /// UTF-8 HTTP data.
    Text(String),
    /// Binary HTTP data.
    Binary(Vec<u8>),
}

impl HttpData {
    /// Attempt to convert this data to a raw String. It is an error if the data
    /// is not valid UTF-8.
    pub fn try_into_string(self) -> Result<String> {
        Ok(match self {
            HttpData::Text(s) => s,
            HttpData::Binary(b) => String::from_utf8(b)?,
        })
    }
}

// We can't just derive PartialEq, because we want to treat structures which
// contain the exact same bytes, albeit using two different representations
// (String vs. byte Vec), as equivalent.
impl PartialEq for HttpData {
    fn eq(&self, other: &HttpData) -> bool {
        (match self {
            HttpData::Text(s) => s.as_bytes(),
            HttpData::Binary(b) => b.as_slice(),
        }) == (match other {
            HttpData::Text(s) => s.as_bytes(),
            HttpData::Binary(b) => b.as_slice(),
        })
    }
}

impl Eq for HttpData {}

impl From<&HeaderValue> for HttpData {
    fn from(value: &HeaderValue) -> HttpData {
        match value.to_str() {
            Ok(s) => HttpData::Text(s.to_owned()),
            Err(_) => HttpData::Binary(value.as_bytes().to_vec()),
        }
    }
}

impl From<&[u8]> for HttpData {
    fn from(bytes: &[u8]) -> Self {
        match std::str::from_utf8(bytes) {
            Ok(text) => HttpData::Text(text.to_owned()),
            Err(_) => HttpData::Binary(bytes.to_vec()),
        }
    }
}

/// A convenient typedef for the structure we store headers in.
pub type HeaderMap = HashMap<String, Vec<HttpData>>;

/// ResponseMetadata stores recorded metadata about an HTTP response.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ResponseMetadata {
    // Stored as u16 to allow serialization.
    status: u16,
    headers: HeaderMap,
}

impl ResponseMetadata {
    /// get_status returns this metadata's HTTP status code.
    pub fn get_status(&self) -> Result<StatusCode> {
        match StatusCode::from_u16(self.status) {
            Err(_) => Err(Error::Internal(format_err!(
                "Invalid ResponseMetadata status code representation {}",
                self.status
            ))),
            Ok(status) => Ok(status),
        }
    }

    /// Returns a reference to the full set of response headers.
    pub fn get_headers(&self) -> &HashMap<String, Vec<HttpData>> {
        &self.headers
    }
}

impl<'a> From<&'a Response> for ResponseMetadata {
    fn from(res: &'a Response) -> Self {
        let mut headers = HashMap::new();
        for (name, value) in res.headers().iter() {
            let value: HttpData = match value.to_str() {
                Ok(s) => HttpData::Text(s.to_owned()),
                Err(_) => HttpData::Binary(value.as_bytes().to_vec()),
            };
            let entry = headers
                .entry(name.as_str().to_owned())
                .or_insert_with(Vec::new);
            (*entry).push(value);
        }

        ResponseMetadata {
            status: res.status().as_u16(),
            headers: headers,
        }
    }
}
