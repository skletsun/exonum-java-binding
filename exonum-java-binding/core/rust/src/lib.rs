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

//! Rust jni helpers for java bindings.
//!
//! All methods catch rust-panic and then rethrow java exception, but because of JNI-specific,
//! some default stub value still should be returned from the Rust side.

#![deny(missing_docs)]
// Function names must follow Java naming for the native functions.
#![allow(non_snake_case)]

extern crate chrono;
pub extern crate exonum;
#[macro_use]
extern crate exonum_derive;
extern crate failure;
pub extern crate jni;
extern crate toml;
#[macro_use]
extern crate log;
extern crate parking_lot;
extern crate protobuf;
extern crate serde;
#[macro_use]
extern crate serde_derive;
pub extern crate serde_json;

#[macro_use]
extern crate lazy_static;

extern crate exonum_testkit;
extern crate exonum_time;
#[cfg(test)]
extern crate tempfile;
extern crate semver;
extern crate futures;

pub use exonum::exonum_merkledb;

pub mod handle;
mod proxy;
mod proto;
mod runtime;
mod storage;
mod testkit;
pub mod utils;

pub use self::handle::{as_handle, cast_handle, drop_handle, to_handle, Handle};
pub use handle::resource_manager::*;
pub use proxy::*;
pub use runtime::services;
pub use runtime::*;
pub use storage::*;
pub use testkit::*;

pub use jni::errors::{Error as JniError, ErrorKind as JniErrorKind, Result as JniResult};
pub use jni::Executor;
