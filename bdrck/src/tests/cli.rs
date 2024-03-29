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

use crate::cli::*;
use crate::error::*;
use std::collections::{HashSet, VecDeque};
use std::io::{Read, Write};

// The write buffer size we preallocate, per instance of `TestStreamBuffers`.
const TEST_WRITE_BUFFER_SIZE_BYTES: usize = 1024 * 100;

/// This structure holds some fake terminal attributes, which the `cli` module
/// can modify via `AbstractStream`, and which we can then inspect in our test.
#[derive(Clone, Debug, Eq, PartialEq)]
struct TestTerminalAttributes {
    on: HashSet<TerminalFlag>,
    off: HashSet<TerminalFlag>,
}

impl TestTerminalAttributes {
    fn new() -> Self {
        TestTerminalAttributes {
            on: [TerminalFlag::Echo].iter().cloned().collect(),
            off: HashSet::new(),
        }
    }

    fn new_specific_state(enabled: &[TerminalFlag], disabled: &[TerminalFlag]) -> Self {
        let mut attrs = Self::new();
        for &f in enabled {
            attrs.enable(f);
        }
        for &f in disabled {
            attrs.disable(f);
        }
        attrs
    }
}

impl Default for TestTerminalAttributes {
    fn default() -> Self {
        Self::new()
    }
}

impl AbstractTerminalAttributes for TestTerminalAttributes {
    fn enable(&mut self, flag: TerminalFlag) {
        self.on.insert(flag);
        self.off.remove(&flag);
    }

    fn disable(&mut self, flag: TerminalFlag) {
        self.on.remove(&flag);
        self.off.insert(flag);
    }
}

/// This holds raw pointers to various bits of text context. This exists so
/// `TestStream` and its reader and writer pieces can access / mutate the test
/// context, while still being able to be consumed (moved) by the `cli` API.
///
/// We're doing this with raw pointers / unsafe because it's hard (impossible?)
/// to accomplish this within Rust's lifetime rules, and after all this is only
/// for testing, so whatever.
#[derive(Clone, Copy)]
struct TestContextPtrs {
    read_attributes_ptr: *mut VecDeque<TestTerminalAttributes>,
    write_attributes_ptr: *mut VecDeque<TestTerminalAttributes>,
    read_ptr: (*const u8, *const u8),
    write_ptr: (*mut u8, *mut u8),
}

/// A `Read` implementation which operates on our test buffer.
struct TestStreamReader {
    ctx: *mut TestContextPtrs,
}

impl Read for TestStreamReader {
    fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
        let (current, end) = unsafe { (*self.ctx).read_ptr };
        let remaining = end as usize - current as usize;
        let to_read = std::cmp::min(remaining, buf.len());
        unsafe {
            std::ptr::copy_nonoverlapping(current, buf.as_mut_ptr(), to_read);
            (*self.ctx).read_ptr = (current.offset(to_read as isize), end);
        }
        Ok(to_read)
    }
}

/// A `Write` implementation which operates on our test buffer.
struct TestStreamWriter {
    ctx: *mut TestContextPtrs,
}

impl Write for TestStreamWriter {
    fn write(&mut self, buf: &[u8]) -> IoResult<usize> {
        let (current, end) = unsafe { (*self.ctx).write_ptr };
        let remaining = end as usize - current as usize;
        let to_write = std::cmp::min(remaining, buf.len());
        if to_write < buf.len() {
            panic!(
                "Attempted to write {} bytes, only {} bytes left in buffer",
                buf.len(),
                remaining
            );
        }
        unsafe {
            std::ptr::copy_nonoverlapping(buf.as_ptr(), current, to_write);
            (*self.ctx).write_ptr = (current.offset(to_write as isize), end);
        }
        Ok(to_write)
    }

    fn flush(&mut self) -> IoResult<()> {
        Ok(())
    }
}

/// An `AbstractStream` implementation, which references a central
/// `TestContext`. Importantly, if you create several streams, they all share
/// the same underlying context.
///
/// It's completely fine and intended to create several `TestStream` instances
/// from a single `TestContext`, and let the `cli` API consume (move) these.
/// You can examine what was done to those streams by examining the
/// `TestContext` after the fact.
///
/// An important consequence of this is that it is not safe to use these across
/// threads; doing so results in undefined behavior (crashes or overwritten
/// data).
struct TestStream {
    isatty: bool,
    support_read: bool,
    support_write: bool,
    ctx: *mut TestContextPtrs,
}

impl TestStream {
    fn get_attributes_ptr(&self) -> *mut VecDeque<TestTerminalAttributes> {
        if self.support_read {
            unsafe { (*self.ctx).read_attributes_ptr }
        } else {
            unsafe { (*self.ctx).write_attributes_ptr }
        }
    }
}

impl AbstractStream for TestStream {
    type Attributes = TestTerminalAttributes;

    fn isatty(&self) -> bool {
        self.isatty
    }

    fn get_attributes(&self) -> IoResult<Self::Attributes> {
        Ok(unsafe { (*self.get_attributes_ptr()).back().unwrap().clone() })
    }

    fn set_attributes(&mut self, attributes: &Self::Attributes) -> IoResult<()> {
        unsafe { (*self.get_attributes_ptr()).push_back(attributes.clone()) };
        Ok(())
    }

    fn as_reader(&self) -> Option<Box<dyn Read>> {
        match self.support_read {
            false => None,
            true => Some(Box::new(TestStreamReader { ctx: self.ctx })),
        }
    }

    fn as_writer(&self) -> Option<Box<dyn Write>> {
        match self.support_write {
            false => None,
            true => Some(Box::new(TestStreamWriter { ctx: self.ctx })),
        }
    }
}

fn attributes_are_default(attributes: &VecDeque<TestTerminalAttributes>) -> bool {
    return attributes.len() == 1
        && *attributes.back().unwrap() == TestTerminalAttributes::default();
}

/// A structure which manages context for a `cli` unit test. This structure
/// provides both `Read` and `Write` streams. Generally speaking, each test
/// will create exactly one of these, and use `as_stream` to get
/// `AbstractStream` instances to pass into the `cli` API.
struct TestContext {
    read_attributes_over_time: Box<VecDeque<TestTerminalAttributes>>,
    write_attributes_over_time: Box<VecDeque<TestTerminalAttributes>>,
    // This field is used via a pointer into it, but because we're doing
    // `unsafe` weirdness the compiler doesn't notice. Suppress the warning.
    #[allow(dead_code)]
    read_buffer: Vec<u8>,
    write_buffer: Vec<u8>,
    ctx: Box<TestContextPtrs>,
}

impl TestContext {
    fn new(read_input: &str) -> Self {
        let mut read_attributes_over_time: Box<VecDeque<TestTerminalAttributes>> =
            Box::new(vec![TestTerminalAttributes::default()].into());
        let mut write_attributes_over_time = read_attributes_over_time.clone();
        let read_buffer = read_input.as_bytes().to_vec();
        let mut write_buffer = vec![0; TEST_WRITE_BUFFER_SIZE_BYTES];

        let ctx = Box::new(TestContextPtrs {
            read_attributes_ptr: read_attributes_over_time.as_mut(),
            write_attributes_ptr: write_attributes_over_time.as_mut(),
            read_ptr: (read_buffer.as_ptr(), unsafe {
                read_buffer.as_ptr().offset(read_buffer.len() as isize)
            }),
            write_ptr: (write_buffer.as_mut_ptr(), unsafe {
                write_buffer
                    .as_mut_ptr()
                    .offset(write_buffer.len() as isize)
            }),
        });

        TestContext {
            read_attributes_over_time: read_attributes_over_time,
            write_attributes_over_time: write_attributes_over_time,
            read_buffer: read_buffer,
            write_buffer: write_buffer,
            ctx: ctx,
        }
    }

    fn has_default_attributes(&self) -> bool {
        attributes_are_default(&self.read_attributes_over_time)
            && attributes_are_default(&self.write_attributes_over_time)
    }

    fn as_stream(&mut self, isatty: bool, support_read: bool, support_write: bool) -> TestStream {
        if support_read && support_write {
            panic!("Test streams must be either read streams or write streams.");
        }
        TestStream {
            support_read: support_read,
            support_write: support_write,
            isatty: isatty,
            ctx: self.ctx.as_mut(),
        }
    }

    fn write_buffer_as_str(&self) -> Result<&str> {
        let len = self.write_buffer.iter().take_while(|&&b| b != 0).count();
        Ok(std::str::from_utf8(&self.write_buffer[0..len])?)
    }
}

/// Create a standard test context, which works for "successful" tests. If you
/// want to test an error / edge case, you might need to do this manually
/// instead.
///
/// Returns a tuple of (context, input stream, output stream).
fn create_normal_test_context(read_input: &str) -> (TestContext, TestStream, TestStream) {
    let mut ctx = TestContext::new(read_input);
    let is = ctx.as_stream(
        /*isatty=*/ true, /*support_read=*/ true, /*support_write=*/ false,
    );
    let os = ctx.as_stream(
        /*isatty=*/ true, /*support_read=*/ false, /*support_write=*/ true,
    );
    (ctx, is, os)
}

const TEST_PROMPT: &'static str = "Test Prompt: ";
const TEST_CONTINUE_DESCRIPTION: &'static str = "Some test thing is about to happen.";

#[test]
fn test_real_terminal_attributes() {
    crate::init().unwrap();

    let mut attrs = TerminalAttributes::new_empty();

    // Empty, so nothing should be enabled.
    assert!(!attrs.is_enabled(TerminalFlag::Echo));
    assert!(!attrs.is_enabled(TerminalFlag::EchoNewlines));

    // If we enable them, then they should actually be enabled.
    attrs.enable(TerminalFlag::Echo);
    assert!(attrs.is_enabled(TerminalFlag::Echo));
    assert!(!attrs.is_enabled(TerminalFlag::EchoNewlines));
    attrs.enable(TerminalFlag::EchoNewlines);
    assert!(attrs.is_enabled(TerminalFlag::Echo));
    assert!(attrs.is_enabled(TerminalFlag::EchoNewlines));

    // And, finally, if we disable them, they should actually be disabled.
    attrs.disable(TerminalFlag::Echo);
    assert!(!attrs.is_enabled(TerminalFlag::Echo));
    assert!(attrs.is_enabled(TerminalFlag::EchoNewlines));
    attrs.disable(TerminalFlag::EchoNewlines);
    assert!(!attrs.is_enabled(TerminalFlag::Echo));
    assert!(!attrs.is_enabled(TerminalFlag::EchoNewlines));
}

#[test]
fn test_input_stream_must_be_a_tty() {
    crate::init().unwrap();

    let mut ctx = TestContext::new("");
    let is = ctx.as_stream(
        /*isatty=*/ false, /*support_read=*/ true, /*support_write=*/ false,
    );
    let os = ctx.as_stream(
        /*isatty=*/ true, /*support_read=*/ false, /*support_write=*/ true,
    );
    assert!(prompt_for_string(is, os, TEST_PROMPT, /*is_sensitive=*/ false).is_err());
}

#[test]
fn test_output_stream_must_be_a_tty() {
    crate::init().unwrap();

    let mut ctx = TestContext::new("");
    let is = ctx.as_stream(
        /*isatty=*/ true, /*support_read=*/ true, /*support_write=*/ false,
    );
    let os = ctx.as_stream(
        /*isatty=*/ false, /*support_read=*/ false, /*support_write=*/ true,
    );
    assert!(prompt_for_string(is, os, TEST_PROMPT, /*is_sensitive=*/ false).is_err());
}

#[test]
fn test_input_stream_must_support_read() {
    crate::init().unwrap();

    let mut ctx = TestContext::new("");
    let is = ctx.as_stream(
        /*isatty=*/ true, /*support_read=*/ false, /*support_write=*/ false,
    );
    let os = ctx.as_stream(
        /*isatty=*/ true, /*support_read=*/ false, /*support_write=*/ true,
    );
    assert!(prompt_for_string(is, os, TEST_PROMPT, /*is_sensitive=*/ false).is_err());
}

#[test]
fn test_output_stream_must_support_write() {
    crate::init().unwrap();

    let mut ctx = TestContext::new("");
    let is = ctx.as_stream(
        /*isatty=*/ true, /*support_read=*/ true, /*support_write=*/ false,
    );
    let os = ctx.as_stream(
        /*isatty=*/ true, /*support_read=*/ false, /*support_write=*/ false,
    );
    assert!(prompt_for_string(is, os, TEST_PROMPT, /*is_sensitive=*/ false).is_err());
}

#[test]
fn test_prompt_for_string() {
    crate::init().unwrap();

    let (ctx, is, os) = create_normal_test_context("foobar\n");
    let result = prompt_for_string(is, os, TEST_PROMPT, /*is_sensitive=*/ false).unwrap();

    assert_eq!("foobar", result);
    assert!(ctx.has_default_attributes());
    assert_eq!(TEST_PROMPT, ctx.write_buffer_as_str().unwrap());
}

#[test]
fn test_prompt_for_string_sensitive() {
    crate::init().unwrap();

    let (ctx, is, os) = create_normal_test_context("foobar\n");
    let result = prompt_for_string(is, os, TEST_PROMPT, /*is_sensitive=*/ true).unwrap();

    assert_eq!("foobar", result);
    let expected_read_attributes_over_time: VecDeque<TestTerminalAttributes> = vec![
        TestTerminalAttributes::default(),
        TestTerminalAttributes::new_specific_state(
            /*enabled=*/ &[TerminalFlag::EchoNewlines],
            /*disabled=*/ &[TerminalFlag::Echo],
        ),
        TestTerminalAttributes::default(),
    ]
    .into();
    assert_eq!(
        expected_read_attributes_over_time,
        *ctx.read_attributes_over_time
    );
    assert!(attributes_are_default(
        ctx.write_attributes_over_time.as_ref()
    ));
    assert_eq!(TEST_PROMPT, ctx.write_buffer_as_str().unwrap());
}

#[test]
fn test_prompt_for_string_confirm() {
    crate::init().unwrap();

    let (ctx, is, os) = create_normal_test_context("foobar\nfoobar\n");
    let result = prompt_for_string_confirm(is, os, TEST_PROMPT, /*is_sensitive=*/ false).unwrap();

    assert_eq!("foobar", result);
    assert!(ctx.has_default_attributes());
    assert_eq!(
        format!("{}Confirm: ", TEST_PROMPT),
        ctx.write_buffer_as_str().unwrap()
    );
}

#[test]
fn test_prompt_for_string_confirm_mismatch() {
    crate::init().unwrap();

    let (ctx, is, os) = create_normal_test_context("foo\nbar\nfoo\nfoo\n");
    let result = prompt_for_string_confirm(is, os, TEST_PROMPT, /*is_sensitive=*/ false).unwrap();

    assert_eq!("foo", result);
    assert!(ctx.has_default_attributes());
    assert_eq!(
        format!("{}Confirm: {}Confirm: ", TEST_PROMPT, TEST_PROMPT),
        ctx.write_buffer_as_str().unwrap()
    );
}

#[test]
fn test_prompt_for_string_confirm_sensitive() {
    crate::init().unwrap();

    let (ctx, is, os) = create_normal_test_context("foobar\nfoobar\n");
    let result = prompt_for_string_confirm(is, os, TEST_PROMPT, /*is_sensitive=*/ true).unwrap();

    assert_eq!("foobar", result);
    let expected_read_attributes_over_time: VecDeque<TestTerminalAttributes> = vec![
        TestTerminalAttributes::default(),
        TestTerminalAttributes::new_specific_state(
            /*enabled=*/ &[TerminalFlag::EchoNewlines],
            /*disabled=*/ &[TerminalFlag::Echo],
        ),
        TestTerminalAttributes::default(),
        TestTerminalAttributes::new_specific_state(
            /*enabled=*/ &[TerminalFlag::EchoNewlines],
            /*disabled=*/ &[TerminalFlag::Echo],
        ),
        TestTerminalAttributes::default(),
    ]
    .into();
    assert_eq!(
        expected_read_attributes_over_time,
        *ctx.read_attributes_over_time
    );
    assert!(attributes_are_default(
        ctx.write_attributes_over_time.as_ref()
    ));
    assert_eq!(
        format!("{}Confirm: ", TEST_PROMPT),
        ctx.write_buffer_as_str().unwrap()
    );
}

#[test]
fn test_maybe_prompted_string() {
    crate::init().unwrap();

    let (ctx, is, os) = create_normal_test_context("foobar\n");
    let mps = MaybePromptedString::new(
        /*provided=*/ None,
        is,
        os,
        TEST_PROMPT,
        /*is_sensitive=*/ false,
        /*confirm=*/ false,
    )
    .unwrap();

    assert!(!mps.was_provided());
    assert_eq!("foobar", mps.into_inner());
    assert!(ctx.has_default_attributes());
    assert_eq!(TEST_PROMPT, ctx.write_buffer_as_str().unwrap());
}

#[test]
fn test_maybe_prompted_string_provided() {
    crate::init().unwrap();

    let (ctx, is, os) = create_normal_test_context("");
    let mps = MaybePromptedString::new(
        /*provided=*/ Some("foobar"),
        is,
        os,
        TEST_PROMPT,
        /*is_sensitive=*/ false,
        /*confirm=*/ false,
    )
    .unwrap();

    assert!(mps.was_provided());
    assert_eq!("foobar", mps.into_inner());
    assert!(ctx.has_default_attributes());
    assert!(ctx.write_buffer_as_str().unwrap().is_empty());
}

#[test]
fn test_maybe_prompted_string_sensitive() {
    crate::init().unwrap();

    let (ctx, is, os) = create_normal_test_context("foobar\n");
    let mps = MaybePromptedString::new(
        /*provided=*/ None,
        is,
        os,
        TEST_PROMPT,
        /*is_sensitive=*/ true,
        /*confirm=*/ false,
    )
    .unwrap();

    assert!(!mps.was_provided());
    assert_eq!("foobar", mps.into_inner());
    let expected_read_attributes_over_time: VecDeque<TestTerminalAttributes> = vec![
        TestTerminalAttributes::default(),
        TestTerminalAttributes::new_specific_state(
            /*enabled=*/ &[TerminalFlag::EchoNewlines],
            /*disabled=*/ &[TerminalFlag::Echo],
        ),
        TestTerminalAttributes::default(),
    ]
    .into();
    assert_eq!(
        expected_read_attributes_over_time,
        *ctx.read_attributes_over_time
    );
    assert!(attributes_are_default(
        ctx.write_attributes_over_time.as_ref()
    ));
    assert_eq!(TEST_PROMPT, ctx.write_buffer_as_str().unwrap());
}

#[test]
fn test_maybe_prompted_string_confirm() {
    crate::init().unwrap();

    let (ctx, is, os) = create_normal_test_context("foobar\nfoobar\n");
    let mps = MaybePromptedString::new(
        /*provided=*/ None,
        is,
        os,
        TEST_PROMPT,
        /*is_sensitive=*/ false,
        /*confirm=*/ true,
    )
    .unwrap();

    assert!(!mps.was_provided());
    assert_eq!("foobar", mps.into_inner());
    assert!(ctx.has_default_attributes());
    assert_eq!(
        format!("{}Confirm: ", TEST_PROMPT),
        ctx.write_buffer_as_str().unwrap()
    );
}

#[test]
fn test_maybe_prompted_string_confirm_mismatch() {
    crate::init().unwrap();

    let (ctx, is, os) = create_normal_test_context("foo\nbar\nfoo\nfoo\n");
    let mps = MaybePromptedString::new(
        /*provided=*/ None,
        is,
        os,
        TEST_PROMPT,
        /*is_sensitive=*/ false,
        /*confirm=*/ true,
    )
    .unwrap();

    assert!(!mps.was_provided());
    assert_eq!("foo", mps.into_inner());
    assert!(ctx.has_default_attributes());
    assert_eq!(
        format!("{}Confirm: {}Confirm: ", TEST_PROMPT, TEST_PROMPT),
        ctx.write_buffer_as_str().unwrap()
    );
}

#[test]
fn test_continue_confirmation_y() {
    crate::init().unwrap();

    let (ctx, is, os) = create_normal_test_context("y\n");
    let result = continue_confirmation(is, os, TEST_CONTINUE_DESCRIPTION).unwrap();

    assert!(result);
    assert!(ctx.has_default_attributes());
    assert_eq!(
        format!("{}Continue? [Yes/No] ", TEST_CONTINUE_DESCRIPTION),
        ctx.write_buffer_as_str().unwrap()
    );
}

#[test]
fn test_continue_confirmation_yes() {
    crate::init().unwrap();

    let (ctx, is, os) = create_normal_test_context("yes\n");
    let result = continue_confirmation(is, os, TEST_CONTINUE_DESCRIPTION).unwrap();

    assert!(result);
    assert!(ctx.has_default_attributes());
    assert_eq!(
        format!("{}Continue? [Yes/No] ", TEST_CONTINUE_DESCRIPTION),
        ctx.write_buffer_as_str().unwrap()
    );
}

#[test]
fn test_continue_confirmation_yes_any_case() {
    crate::init().unwrap();

    let (ctx, is, os) = create_normal_test_context("YeS\n");
    let result = continue_confirmation(is, os, TEST_CONTINUE_DESCRIPTION).unwrap();

    assert!(result);
    assert!(ctx.has_default_attributes());
    assert_eq!(
        format!("{}Continue? [Yes/No] ", TEST_CONTINUE_DESCRIPTION),
        ctx.write_buffer_as_str().unwrap()
    );
}

#[test]
fn test_continue_confirmation_n() {
    crate::init().unwrap();

    let (ctx, is, os) = create_normal_test_context("n\n");
    let result = continue_confirmation(is, os, TEST_CONTINUE_DESCRIPTION).unwrap();

    assert!(!result);
    assert!(ctx.has_default_attributes());
    assert_eq!(
        format!("{}Continue? [Yes/No] ", TEST_CONTINUE_DESCRIPTION),
        ctx.write_buffer_as_str().unwrap()
    );
}

#[test]
fn test_continue_confirmation_no() {
    crate::init().unwrap();

    let (ctx, is, os) = create_normal_test_context("no\n");
    let result = continue_confirmation(is, os, TEST_CONTINUE_DESCRIPTION).unwrap();

    assert!(!result);
    assert!(ctx.has_default_attributes());
    assert_eq!(
        format!("{}Continue? [Yes/No] ", TEST_CONTINUE_DESCRIPTION),
        ctx.write_buffer_as_str().unwrap()
    );
}

#[test]
fn test_continue_confirmation_no_any_case() {
    crate::init().unwrap();

    let (ctx, is, os) = create_normal_test_context("nO\n");
    let result = continue_confirmation(is, os, TEST_CONTINUE_DESCRIPTION).unwrap();

    assert!(!result);
    assert!(ctx.has_default_attributes());
    assert_eq!(
        format!("{}Continue? [Yes/No] ", TEST_CONTINUE_DESCRIPTION),
        ctx.write_buffer_as_str().unwrap()
    );
}

#[test]
fn test_continue_confirmation_bad_input() {
    crate::init().unwrap();

    let (ctx, is, os) = create_normal_test_context("foo\nYes\n");
    let result = continue_confirmation(is, os, TEST_CONTINUE_DESCRIPTION).unwrap();

    assert!(result);
    assert!(ctx.has_default_attributes());
    assert_eq!(
        format!(
            "{}Continue? [Yes/No] Invalid response 'foo'.\n{}Continue? [Yes/No] ",
            TEST_CONTINUE_DESCRIPTION, TEST_CONTINUE_DESCRIPTION
        ),
        ctx.write_buffer_as_str().unwrap()
    );
}
