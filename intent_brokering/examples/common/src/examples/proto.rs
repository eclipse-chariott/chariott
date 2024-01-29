// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

mod intent_brokering {
    pub mod common {
        pub use intent_brokering_proto::common as v1;
    }
}

mod examples {
    pub mod detection {
        pub mod v1 {
            // see https://github.com/hyperium/tonic/issues/1056
            // and https://github.com/tokio-rs/prost/issues/661#issuecomment-1156606409
            // why we use allow derive_partial_eq_without_eq
            #![allow(clippy::derive_partial_eq_without_eq)]
            tonic::include_proto!("examples.detection.v1");
        }
    }
}

pub use examples::detection::v1 as detection;
