use super::{
    AbiBody, AbiFunction, AbiMetadata, AbiRoot, SCHEMA_VERSION, Schema, ensure_current_version,
};
use schemars::SchemaGenerator;
use serde::{Deserialize, Serialize};
use std::fmt;

/// Core ABI information, with schema version and identity hash.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ChunkedAbiEntry {
    /// Semver of the ABI schema format.
    #[serde(deserialize_with = "ensure_current_version")]
    pub schema_version: String,
    #[serde(flatten)]
    pub body: AbiBody,
}

impl ChunkedAbiEntry {
    pub fn new(functions: Vec<AbiFunction>, root_schema: Schema) -> ChunkedAbiEntry {
        Self {
            schema_version: SCHEMA_VERSION.to_string(),
            body: AbiBody {
                functions,
                root_schema,
            },
        }
    }

    pub fn combine<I: IntoIterator<Item = ChunkedAbiEntry>>(
        entries: I,
    ) -> Result<ChunkedAbiEntry, AbiCombineError> {
        let mut schema_version = None;
        let mut functions = Vec::<AbiFunction>::new();

        let mut schema_gen = SchemaGenerator::default();
        let definitions = schema_gen.definitions_mut();

        let mut unexpected_versions = std::collections::BTreeSet::new();

        for entry in entries {
            if let Some(ref schema_version) = schema_version {
                // should probably only disallow major version mismatch
                if schema_version != &entry.schema_version {
                    unexpected_versions.insert(entry.schema_version.clone());
                    continue;
                }
            } else {
                schema_version = Some(entry.schema_version);
            }

            // Update resulting JSON Schema
            // In schemars 1.x, definitions are stored in $defs within the schema value
            let schema_value = entry.body.root_schema.to_value();
            if let serde_json::Value::Object(map) = schema_value {
                if let Some(serde_json::Value::Object(defs)) = map.get("$defs") {
                    for (k, v) in defs {
                        definitions.insert(
                            k.clone().into(),
                            serde_json::from_value(v.clone()).unwrap_or_default(),
                        );
                    }
                }
            }

            // Update resulting function list
            functions.extend(entry.body.functions);
        }

        if !unexpected_versions.is_empty() {
            return Err(AbiCombineError {
                kind: AbiCombineErrorKind::SchemaVersionConflict {
                    expected: schema_version.unwrap(),
                    found: unexpected_versions.into_iter().collect(),
                },
            });
        }

        // Sort the function list for readability
        functions.sort_by(|x, y| x.name.cmp(&y.name));

        Ok(ChunkedAbiEntry {
            schema_version: schema_version.unwrap(),
            body: AbiBody {
                functions,
                root_schema: schema_gen.into_root_schema_for::<String>(),
            },
        })
    }

    pub fn into_abi_root(self, metadata: AbiMetadata) -> AbiRoot {
        AbiRoot {
            schema_version: self.schema_version,
            metadata,
            body: self.body,
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
