// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

// see https://github.com/hyperium/tonic/issues/1056
// and https://github.com/tokio-rs/prost/issues/661#issuecomment-1156606409
// why we use allow derive_partial_eq_without_eq
#![allow(clippy::derive_partial_eq_without_eq)]

mod chariott {
    pub mod common {
        pub mod v1 {
            tonic::include_proto!("chariott.common.v1");
        }
    }
    pub mod provider {
        pub mod v1 {
            tonic::include_proto!("chariott.provider.v1");
        }
    }
    pub mod runtime {
        pub mod v1 {
            tonic::include_proto!("chariott.runtime.v1");
        }
    }
    pub mod streaming {
        pub mod v1 {
            tonic::include_proto!("chariott.streaming.v1");
        }
    }
}

pub use chariott::common::v1 as common;
pub use chariott::provider::v1 as provider;
pub use chariott::runtime::v1 as runtime;
pub use chariott::streaming::v1 as streaming;
