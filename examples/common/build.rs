// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

use std::{error::Error, path::Path};
use tonic_build::configure;

fn main() -> Result<(), Box<dyn Error>> {
    configure().compile(
        &[Path::new("../applications/proto/examples/detection/v1/detection.proto")],
        &[Path::new("../../proto/"), Path::new("../applications/proto/examples/detection/v1/")],
    )?;

    Ok(())
}
