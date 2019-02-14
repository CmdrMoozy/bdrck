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
use crate::http::types::HeaderMap;
use failure::format_err;
use reqwest::header;
use reqwest::Url;
use std::collections::HashMap;

/// This function parses the contents of the HTTP "link" header. The result is
/// a map from rel to URL(s). Note that the returned map may be empty - this is
/// not an error.
pub fn get_links(headers: &HeaderMap) -> Result<HashMap<String, Vec<Url>>> {
    let mut urls = HashMap::new();
    if let Some(values) = headers.get(header::LINK.as_str()) {
        for value in values {
            let mut value = value.clone().try_into_string()?;
            while !value.is_empty() {
                // Parse the URL from the front of this string.
                if !value.starts_with('<') {
                    return Err(Error::InvalidArgument(format_err!(
                        "Invalid link header value format: '{}'",
                        value
                    )));
                }
                let url_end = match value.find('>') {
                    None => {
                        return Err(Error::InvalidArgument(format_err!(
                            "Invalid link header value format: '{}'",
                            value
                        )));
                    }
                    Some(idx) => idx,
                };
                let url: Url = (&value[1..url_end]).parse()?;
                value.replace_range(0..url_end + 1, "");

                // Parse the rel string.
                if !value.starts_with("; rel=\"") {
                    return Err(Error::InvalidArgument(format_err!(
                        "Invalid link header value format: '{}'",
                        value
                    )));
                }
                value.replace_range(0..7, "");
                let rel_end = match value.find("\"") {
                    None => {
                        return Err(Error::InvalidArgument(format_err!(
                            "Invalid link header value format: '{}'",
                            value
                        )));
                    }
                    Some(idx) => idx,
                };
                let rel = (&value[..rel_end]).to_owned();
                value.replace_range(..rel_end + 1, "");
                if value.starts_with(", ") {
                    value.replace_range(..2, "");
                }

                // Insert this URL into the map.
                let entry = urls.entry(rel).or_insert_with(Vec::new);
                (*entry).push(url);
            }
        }
    }
    Ok(urls)
}
