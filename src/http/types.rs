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
use reqwest::{Response, StatusCode};

/// ResponseMetadata stores recorded metadata about an HTTP response.
#[derive(Clone, Deserialize, Serialize)]
pub struct ResponseMetadata {
    status: u16,
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
}

impl<'a> From<&'a Response> for ResponseMetadata {
    fn from(res: &'a Response) -> Self {
        ResponseMetadata {
            status: res.status().as_u16(),
        }
    }
}
