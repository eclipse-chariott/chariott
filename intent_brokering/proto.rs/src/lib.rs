// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

// see https://github.com/hyperium/tonic/issues/1056
// and https://github.com/tokio-rs/prost/issues/661#issuecomment-1156606409
// why we use allow derive_partial_eq_without_eq
#![allow(clippy::derive_partial_eq_without_eq)]

pub mod intent_brokering {
    pub mod common {
        pub mod v1 {
            tonic::include_proto!("intent_brokering.common.v1");

            // Re-exporting these types under a different name makes it more
            // ergonomic in scenarios where both the "outer" (*Message) and
            // "inner" (*Enum) type is needed, without needing to add qualifiers
            // for the two types.
            pub use fulfillment::Fulfillment as FulfillmentEnum;
            pub use intent::Intent as IntentEnum;
            pub use value::Value as ValueEnum;
            pub use Fulfillment as FulfillmentMessage;
            pub use Intent as IntentMessage;
            pub use Value as ValueMessage;
        }
    }
    pub mod provider {
        pub mod v1 {
            tonic::include_proto!("intent_brokering.provider.v1");
        }
    }
    pub mod runtime {
        pub mod v1 {
            tonic::include_proto!("intent_brokering.runtime.v1");
        }
    }
    pub mod streaming {
        pub mod v1 {
            tonic::include_proto!("intent_brokering.streaming.v1");
        }
    }
}

pub use intent_brokering::common::v1 as common;
pub use intent_brokering::provider::v1 as provider;
pub use intent_brokering::runtime::v1 as runtime;
pub use intent_brokering::streaming::v1 as streaming;
