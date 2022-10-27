// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

//! In Memory Key Value Store
//!
//! Provides an in memory key value store that can be used to store and retrieve
//! values. The store can be observed by providing an observer that is called
//! on each field update.
//!
//! **Note:** This is implementation is not thread safe. The user of this
//! library is responsible for ensuring thread safety.

pub mod key_value_store;
pub use key_value_store::*;
