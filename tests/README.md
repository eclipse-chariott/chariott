# E2E Tests

The E2E tests are executed using Chariott and the KV App. Test files are
suffixed with `e2e.rs`.

To run the tests, you can either use the CI, or run the tests locally.

## Running the tests locally

Assuming the current working directory is the project root, run:

```sh
cargo build --release -p chariott -p kv-app
CHARIOTT_REGISTRY_TTL_SECS=7 ./target/release/chariott &
./target/release/kv-app &
CHARIOTT_REGISTRY_TTL_SECS=7 cargo test --test '*e2e'
```

If you want to display debug logs, make sure to set the log level via the `RUST_LOG` environment variable.

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
running unit tests, e.g.:

```toml
[[test]]
name = "foo-e2e"
test = false

[[test]]
name = "bar-e2e"
test = false
```
