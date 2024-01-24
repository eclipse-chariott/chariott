use std::{env, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Tell cargo to create a configuration key value pair for us.
    // This will be used to determine if we're building for debug or release.
    // see https://doc.rust-lang.org/cargo/reference/build-scripts.html#cargorustc-cfgkeyvalue
    if let Ok(profile) = env::var("PROFILE") {
        println!("cargo:rustc-cfg=build={:?}", profile);
    }

    let out_dir = PathBuf::from(env::var("OUT_DIR")?);

    tonic_build::configure()
        .build_client(true)
        .build_server(true)
        .file_descriptor_set_path(out_dir.join("descriptor.bin"))
        .compile(
            &[
                "./proto/intent_brokering/runtime/v1/runtime.proto",
                "./proto/intent_brokering/provider/v1/provider.proto",
            ],
            &["./proto/"],
        )?;

    Ok(())
}
