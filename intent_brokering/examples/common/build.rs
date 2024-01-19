// Copyright (c) Microsoft Corporation.
// Licensed under the MIT license.
// SPDX-License-Identifier: MIT

use std::{error::Error, path::Path};
use tonic_build::configure;

fn main() -> Result<(), Box<dyn Error>> {
    configure().compile(
        &[Path::new("../applications/proto/examples/detection/v1/detection.proto")],
        &[Path::new("../../proto/"), Path::new("../applications/proto/examples/detection/v1/")],
    )?;

    Ok(())
}
