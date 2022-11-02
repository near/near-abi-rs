## [Unreleased]

## [0.3.0] - 2022-11-02

### Added
- Derived `schemars::JsonSchema` for ABI models. https://github.com/near/near-abi-rs/pull/11, https://github.com/near/near-abi-rs/pull/16, https://github.com/near/near-abi-rs/pull/19

### Fixed
- Skip `wasm_hash` serialization if it is empty. https://github.com/near/near-abi-rs/pull/17

### Changed
- Replaced `is_view`, `is_init`, `is_payable`, `is_private` function fields with `kind` and `modifiers`. https://github.com/near/near-abi-rs/pull/20

## [0.2.0] - 2022-09-21

### Changed
- Included optional build information into metadata. https://github.com/near/near-abi-rs/pull/8
- Consolidated function parameters' serialization type into one place per function. https://github.com/near/near-abi-rs/pull/9

[unreleased]: https://github.com/near/near-abi-rs/compare/v0.2.0...HEAD
[0.2.0]: https://github.com/near/near-abi-rs/compare/v0.1.0-pre.0...v0.2.0
[0.1.0-pre.0]: https://github.com/near/near-abi-rs/releases/tag/v0.1.0-pre.0
