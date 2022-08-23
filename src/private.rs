use std::fmt;

use super::{AbiEntry, AbiFunction, AbiMetadata, AbiRoot, RootSchema, SCHEMA_VERSION};

pub use schemars;
use serde::{Deserialize, Serialize};

/// Core ABI information, with schema version and identity hash.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ChunkedAbiEntry {
    /// Semver of the ABI schema format.
    pub abi_schema_version: String,
    #[serde(flatten)]
    pub abi: AbiEntry,
}

impl ChunkedAbiEntry {
    pub fn new(functions: Vec<AbiFunction>, root_schema: RootSchema) -> ChunkedAbiEntry {
        Self {
            abi_schema_version: SCHEMA_VERSION.to_string(),
            abi: AbiEntry {
                functions,
                root_schema,
            },
        }
    }

    pub fn combine<I: IntoIterator<Item = ChunkedAbiEntry>>(
        entries: I,
    ) -> Result<ChunkedAbiEntry, AbiCombineError> {
        let mut abi_schema_version = None;
        let mut functions = Vec::<AbiFunction>::new();

        let mut gen = schemars::gen::SchemaGenerator::default();
        let definitions = gen.definitions_mut();

        let mut unexpected_versions = std::collections::HashSet::new();

        for entry in entries {
            if let Some(ref abi_schema_version) = abi_schema_version {
                // should probably only disallow major version mismatch
                if abi_schema_version != &entry.abi_schema_version {
                    unexpected_versions.insert(entry.abi_schema_version.clone());
                    continue;
                }
            } else {
                abi_schema_version = Some(entry.abi_schema_version);
            }

            // Update resulting JSON Schema
            definitions.extend(entry.abi.root_schema.definitions.to_owned());

            // Update resulting function list
            functions.extend(entry.abi.functions);
        }

        if !unexpected_versions.is_empty() {
            return Err(AbiCombineError {
                kind: AbiCombineErrorKind::SchemaVersionConflict {
                    expected: abi_schema_version.unwrap(),
                    found: unexpected_versions.into_iter().collect(),
                },
            });
        }

        // Sort the function list for readability
        functions.sort_by(|x, y| x.name.cmp(&y.name));

        Ok(ChunkedAbiEntry {
            abi_schema_version: abi_schema_version.unwrap(),
            abi: AbiEntry {
                functions,
                root_schema: gen.into_root_schema_for::<String>(),
            },
        })
    }

    pub fn into_abi_root(self, metadata: AbiMetadata) -> AbiRoot {
        AbiRoot {
            abi_schema_version: self.abi_schema_version,
            metadata,
            abi: self.abi,
        }
    }
}

#[derive(Eq, Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct AbiCombineError {
    #[serde(flatten)]
    kind: AbiCombineErrorKind,
}

impl AbiCombineError {
    pub fn kind(&self) -> &AbiCombineErrorKind {
        &self.kind
    }
}

impl std::error::Error for AbiCombineError {}
impl fmt::Display for AbiCombineError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.kind.fmt(f)
    }
}

#[derive(Eq, Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AbiCombineErrorKind {
    SchemaVersionConflict {
        expected: String,
        found: Vec<String>,
    },
}

impl fmt::Display for AbiCombineErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::SchemaVersionConflict { expected, found } => format!(
                "ABI schema version conflict: expected {}, found {}",
                expected,
                found.join(", ")
            )
            .fmt(f),
        }
    }
}
