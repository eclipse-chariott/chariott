// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

//! # Overview
//! Common code (types and functions) for the different sdvlib crates.
//!
//! # Usage
//! Add this crate as a dependency and then `use` from your code.
//! Note: This is a lib-only crate.
//!
//! # Getting started
//! In order to get started, reference this library in your Cargo.toml
//!
//! ```toml
//! chariott_common = { path = "../common/" }
//! ```
//!

/// Generic error handling
pub mod error;

/// Extension traits
pub mod ext;

/// Configuration related utilites
pub mod config;

/// Integration of the event sub-system with the gRPC streaming contract.
pub mod streaming_ess;

/// Protobuf generated files.
pub mod proto;

/// Query utilities
pub mod query;

/// Graceful shutdown helpers
pub mod shutdown;

/// Tokio utilities
pub mod tokio_runtime_fork;
