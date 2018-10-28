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

/// client provides a simple HTTP client trait and implementation, based upon
/// reqwest.
pub mod client;
/// recording provides structures used to record HTTP sessions, so they can
/// later be replayed and verified in unit tests.
#[cfg(debug_assertions)]
pub mod recording;
/// types defines custom types for modeling HTTP requests / responses.
pub mod types;
