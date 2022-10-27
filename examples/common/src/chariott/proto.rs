// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

pub mod streaming {
    // see https://github.com/hyperium/tonic/issues/1056
    // and https://github.com/tokio-rs/prost/issues/661#issuecomment-1156606409
    // why we use allow derive_partial_eq_without_eq
    #![allow(clippy::derive_partial_eq_without_eq)]
    tonic::include_proto!("chariott.streaming.v1");
}

pub use chariott_common::proto::common;
pub use chariott_common::proto::provider;
pub use chariott_common::proto::runtime as runtime_api;
