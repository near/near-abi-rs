use std::fmt;

use serde::{Deserialize, Serialize};

use super::{AbiEntry, AbiFunction, RootSchema, SCHEMA_VERSION};

/// Core ABI information, with schema version and identity hash.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ChunkedAbiEntry {
    /// Semver of the ABI schema format.
    pub(crate) abi_schema_version: String,
    pub source_hash: u64,
    #[serde(flatten)]
    pub abi: AbiEntry,
}

impl ChunkedAbiEntry {
    pub fn new(
        source_hash: u64,
        functions: Vec<AbiFunction>,
        root_schema: RootSchema,
    ) -> ChunkedAbiEntry {
        Self {
            abi_schema_version: SCHEMA_VERSION.to_string(),
            source_hash,
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
        let mut source_hash = None;

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
            if let Some(ref source_hash) = source_hash {
                if source_hash != &entry.source_hash {
                    return Err(AbiCombineError {
                        kind: AbiCombineErrorKind::SourceConflict,
                    });
                }
            } else {
                source_hash = Some(entry.source_hash);
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
            source_hash: source_hash.unwrap(),
            abi: AbiEntry {
                functions,
                root_schema: gen.into_root_schema_for::<String>(),
            },
        })
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
    SourceConflict,
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
            Self::SourceConflict => "ABI entry source conflict".fmt(f),
        }
    }
}
