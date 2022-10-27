# E2E Tests

The E2E tests are executed using Chariott and the KV App. Test files are
suffixed with `e2e.rs`.

To run the tests, you can either use the CI, or run the tests locally.

## Running the tests locally

1. Run `cargo build --release -p chariott -p kv-app` to build the required projects.
2. Run Chariott and the Key-Value app in the background, using
`./target/release/chariott &` and `./target/release/kv-app &` (assuming you are
in the repository root). If you want to display debug logs, make sure to set the
log level via the `RUST_LOG` environment variable.
3. Execute the tests using `cargo test --test "*e2e"`.

## Running the tests locally with Docker in WSL2

From the project root, run the following command:

```bash
./tests/container-e2e-tests-wsl2.sh .
```

This will build the Chariott and KV App Docker images, and run the tests.

## Adding new tests

When adding new tests, refer to the `store-e2e.rs` to see which components we
reuse for the E2E tests. Make sure to add a configuration section for the test
crate to the `Cargo.toml`, to ensure that E2E tests are not executed when
running unit tests:

```toml
[[test]]
name = "store-e2e"
test = false
```
