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

use crate::spec::*;

fn find_named_spec_works(specs: &Specs, query: &str, expected_name: &str) -> bool {
    specs
        .find_named_spec(query)
        .map_or(false, |s| s.get_name() == expected_name)
}

#[test]
fn test_find_option() {
    let specs = Specs::new(vec![
        Spec::required("foo", "", Some('o'), None),
        Spec::required("bar", "", Some('r'), None),
        Spec::boolean("baz", "", Some('z')),
        Spec::boolean("zab", "", Some('Z')),
        Spec::required("rab", "", Some('R'), None),
        Spec::optional("oof", "", Some('O')),
        Spec::required("foobar", "", Some('f'), None),
        Spec::boolean("barbaz", "", Some('b')),
        Spec::boolean("zabrab", "", Some('B')),
        Spec::optional("raboof", "", Some('F')),
    ])
    .unwrap();

    assert!(find_named_spec_works(&specs, "foo", "foo"));
    assert!(find_named_spec_works(&specs, "o", "foo"));
    assert!(find_named_spec_works(&specs, "bar", "bar"));
    assert!(find_named_spec_works(&specs, "r", "bar"));
    assert!(find_named_spec_works(&specs, "baz", "baz"));
    assert!(find_named_spec_works(&specs, "z", "baz"));
    assert!(find_named_spec_works(&specs, "zab", "zab"));
    assert!(find_named_spec_works(&specs, "Z", "zab"));
    assert!(find_named_spec_works(&specs, "rab", "rab"));
    assert!(find_named_spec_works(&specs, "R", "rab"));
    assert!(find_named_spec_works(&specs, "oof", "oof"));
    assert!(find_named_spec_works(&specs, "O", "oof"));
    assert!(find_named_spec_works(&specs, "foobar", "foobar"));
    assert!(find_named_spec_works(&specs, "f", "foobar"));
    assert!(find_named_spec_works(&specs, "barbaz", "barbaz"));
    assert!(find_named_spec_works(&specs, "b", "barbaz"));
    assert!(find_named_spec_works(&specs, "zabrab", "zabrab"));
    assert!(find_named_spec_works(&specs, "B", "zabrab"));
    assert!(find_named_spec_works(&specs, "raboof", "raboof"));
    assert!(find_named_spec_works(&specs, "F", "raboof"));

    assert!(!find_named_spec_works(&specs, "foo", "bar"));
    assert!(!find_named_spec_works(&specs, "syn", "syn"));
    assert!(!find_named_spec_works(&specs, "s", "syn"));
    assert!(!find_named_spec_works(&specs, "ack", "ack"));
    assert!(!find_named_spec_works(&specs, "a", "ack"));
    assert!(!find_named_spec_works(&specs, "-", "foobar"));
}
