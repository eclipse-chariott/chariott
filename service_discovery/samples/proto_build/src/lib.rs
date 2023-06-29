// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

pub mod hello_world {
    pub mod v1 {
        #![allow(clippy::derive_partial_eq_without_eq)]
        tonic::include_proto!("hello_world");
    }
}
