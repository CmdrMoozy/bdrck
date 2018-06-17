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
use flags::main_impl::*;

#[test]
fn test_handle_result() {
    ::init().unwrap();

    assert_eq!(EXIT_SUCCESS, handle_result::<Error>(Ok(Ok(()))));
    assert_eq!(
        EXIT_FAILURE,
        handle_result::<Error>(Err(Error::Internal(format_err!(
            "arbitrary internal error"
        ))))
    );
    assert_eq!(
        EXIT_FAILURE,
        handle_result(Ok(Err(Error::Unknown(format_err!(
            "arbitrary command error"
        )))))
    );
}
