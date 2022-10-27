//! # Legacy module
//!
//! ABI schema is an early-stage format that often goes through breaking syntactic changes. However,
//! it is usually possible to migrate one schema version to another without the loss of semantics.
//! Legacy module was designed with this in mind and tries to provide user with ability to interpret
//! a wide range of ABI schema versions. The range is a subject to change and is provided on a best
//! effort basis.
//!
//! Currently, versions all the way back to 0.1.0 are supported.

use std::io::Read;

use serde::de::Error;
use serde_json::Value;

mod migration;
mod v0_1;
mod v0_2;

pub fn from_value(abi: Value) -> serde_json::Result<super::AbiRoot> {
    let abi_object = abi
        .as_object()
        .ok_or_else(|| serde_json::Error::custom("expected ABI to be a JSON object"))?;
    let schema_version = abi_object["schema_version"].as_str().ok_or_else(|| {
        serde_json::Error::custom("expected ABI to have a string field named `schema_version`")
    })?;
    let schema_version: semver::Version = schema_version.parse().map_err(|e| {
        serde_json::Error::custom(format!(
            "expected `schema_version` to contain a valid semver string: {}",
            e
        ))
    })?;
    match (schema_version.major, schema_version.minor) {
        (0, 1) => {
            let abi_root: v0_1::AbiRoot = serde_json::from_value(abi)?;
            let abi_root = migration::v0_1_to_v0_2(abi_root);
            Ok(migration::v0_2_to_current(abi_root))
        }
        (0, 2) => {
            let abi_root: v0_2::AbiRoot = serde_json::from_value(abi)?;
            Ok(migration::v0_2_to_current(abi_root))
        }
        (0, 3) => serde_json::from_value(abi),
        _ => Err(serde_json::Error::custom(format!(
            "Unsupported ABI schema version: {}",
            schema_version
        ))),
    }
}

pub fn from_slice(v: &[u8]) -> serde_json::Result<super::AbiRoot> {
    let abi: serde_json::Value = serde_json::from_slice(v)?;
    from_value(abi)
}

pub fn from_str(s: &str) -> serde_json::Result<super::AbiRoot> {
    let abi: serde_json::Value = serde_json::from_str(s)?;
    from_value(abi)
}

pub fn from_reader<R>(rdr: R) -> serde_json::Result<super::AbiRoot>
where
    R: Read,
{
    let abi: serde_json::Value = serde_json::from_reader(rdr)?;
    from_value(abi)
}
