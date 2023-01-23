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

/// fn_instrumentation provides utilities for instrumenting function calls
/// during unit tests.
pub mod fn_instrumentation;
/// http provides testing support for the http submodule.
#[cfg(debug_assertions)]
pub mod http;
/// temp provides utilities for creating temporary files or directories in unit
/// tests.
pub mod temp;
