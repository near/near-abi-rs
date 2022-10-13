# Contributing to near-sdk-rs

Thank you for your interest in contributing to NEAR! We appreciate any type of contribution.

If you have any questions about contributing, or about the project in general, please ask in our [rust-support Discord channel](https://discord.gg/cKRZCqD2b2).

## Development

### Commits

Please use descriptive PR titles. We loosely follow the [conventional commits](https://www.conventionalcommits.org/en/v1.0.0/) style, but this is not a requirement to follow exactly. PRs will be addressed more quickly if it is clear what the intention is.

### Before opening a PR

Ensure the following are satisfied before opening a PR:
- Code is formatted with `rustfmt` by running `cargo fmt`
- Run `clippy`
  - The exact command run by the CI is `cargo clippy --tests -- -Dclippy::all`
- Run tests with `cargo test`
- If you have changed the ABI models' structure:
  - Re-generate metaschema by running `cargo run --package metaschema-gen > metaschema/near-abi-current-schema.json`
  - Make sure that the change is backwards compatible to the previous ABI schema format or bump the `SCHEMA_VERSION`
- Ensure any new functionality is adequately tested
- If any new public types or functions are added, ensure they have appropriate [rustdoc](https://doc.rust-lang.org/rustdoc/what-is-rustdoc.html) documentation
