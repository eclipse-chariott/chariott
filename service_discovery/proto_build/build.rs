// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tonic_build::compile_protos("../proto/chariott/v1/chariott_registry.proto")?;
    tonic_build::compile_protos("../proto/samples/v1/hello_world_service.proto")?;
    Ok(())
}