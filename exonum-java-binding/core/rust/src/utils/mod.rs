// Copyright 2018 The Exonum Team
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Contains util methods which are used by other top-level modules.

#![deny(non_snake_case)]

mod conversion;
mod errors;
mod jni;
pub mod jni_cache;

pub use self::conversion::{
    convert_hash, convert_to_hash, convert_to_string, java_arrays_to_rust, proto_to_java_bytes,
};
pub use self::errors::{
    any_to_string, check_error_on_exception, describe_java_exception, get_and_clear_java_exception,
    panic_on_exception, unwrap_exc_or, unwrap_exc_or_default, unwrap_jni, unwrap_jni_verbose,
};
pub use self::jni::{get_class_name, get_exception_message};

/// Asserts that given closure panics while executed and the resulting error message contains given
/// substring.
pub fn assert_panics<F, R>(err_substring: &str, f: F)
where
    F: FnOnce() -> R,
{
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(f));
    match result {
        Ok(_) => panic!("Panic expected"),
        Err(err) => {
            let err_msg = any_to_string(&err);
            if !err_msg.contains(err_substring) {
                panic!(
                    "Expected a panic message containing \"{}\" but was:\n{}",
                    err_substring, err_msg
                )
            }
        }
    }
}
