// Copyright (c) Microsoft Corporation. All rights reserved.
// Licensed under the MIT license.

use std::{error::Error, path::Path};
use tonic_build::configure;

fn main() -> Result<(), Box<dyn Error>> {
    compile_with_common("../proto/chariott/runtime/v1/runtime.proto")?;
    compile_with_common("../proto/chariott/provider/v1/provider.proto")?;
    compile_with_common("../proto/chariott/streaming/v1/streaming.proto")?;

    Ok(())
}

fn compile_with_common(path: &str) -> Result<(), Box<dyn Error>> {
    configure().compile(&[Path::new(path)], &[Path::new("../proto/")])?;

    Ok(())
}
