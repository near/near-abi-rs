use borsh::schema::{BorshSchemaContainer, Declaration, Definition, Fields, VariantName};
use schemars::schema::{RootSchema, Schema};
use schemars::JsonSchema;
use semver::Version;
use serde::{de, Deserialize, Deserializer, Serialize};
use std::collections::HashMap;

#[doc(hidden)]
#[cfg(feature = "__chunked-entries")]
#[path = "private.rs"]
pub mod __private;

// Keep in sync with SCHEMA_VERSION below.
const SCHEMA_SEMVER: Version = Version {
    major: 0,
    minor: 2,
    patch: 0,
    pre: semver::Prerelease::EMPTY,
    build: semver::BuildMetadata::EMPTY,
};

/// Current version of the ABI schema format.
pub const SCHEMA_VERSION: &str = "0.2.0";

/// Contract ABI.
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AbiRoot {
    /// Semver of the ABI schema format.
    #[serde(deserialize_with = "ensure_current_version")]
    pub schema_version: String,
    /// Metadata information about the contract.
    pub metadata: AbiMetadata,
    /// Core ABI information (functions and types).
    pub body: AbiBody,
}

fn ensure_current_version<'de, D: Deserializer<'de>>(d: D) -> Result<String, D::Error> {
    let unchecked = String::deserialize(d)?;
    let version = Version::parse(&unchecked)
        .map_err(|_| de::Error::custom("expected `schema_version` to be a valid semver value"))?;
    if version.major != SCHEMA_SEMVER.major || version.minor != SCHEMA_SEMVER.minor {
        if version < SCHEMA_SEMVER {
            return Err(de::Error::custom(format!(
                "expected `schema_version` to be ~{}.{}, but got {}: consider re-generating your ABI file with a newer version of SDK and cargo-near",
                SCHEMA_SEMVER.major, SCHEMA_SEMVER.minor, version
            )));
        } else {
            return Err(de::Error::custom(format!(
                "expected `schema_version` to be ~{}.{}, but got {}: consider upgrading near-abi to a newer version",
                SCHEMA_SEMVER.major, SCHEMA_SEMVER.minor, version
            )));
        }
    }
    Ok(unchecked)
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq, Default, JsonSchema)]
pub struct BuildInfo {
    /// The compiler (versioned) that was used to build the contract.
    pub compiler: String,
    /// The build tool (versioned) that was used to build the contract.
    pub builder: String,
    /// The docker image (versioned) where the contract was built.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, Eq, Default, JsonSchema)]
pub struct AbiMetadata {
    /// The name of the smart contract.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// The version of the smart contract.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// The authors of the smart contract.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub authors: Vec<String>,
    /// The information about how this contract was built.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub build: Option<BuildInfo>,
    /// The SHA-256 hash of the contract WASM code in Base58 format.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub wasm_hash: Option<String>,
    /// Other arbitrary metadata.
    #[serde(default, flatten, skip_serializing_if = "HashMap::is_empty")]
    pub other: HashMap<String, String>,
}

/// Core ABI information.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AbiBody {
    /// ABIs of all contract's functions.
    pub functions: Vec<AbiFunction>,
    /// Root JSON Schema containing all types referenced in the functions.
    pub root_schema: RootSchema,
}

/// ABI of a single function.
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AbiFunction {
    pub name: String,
    /// Human-readable documentation parsed from the source file.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub doc: Option<String>,
    /// Whether function does not modify the state.
    #[serde(default, skip_serializing_if = "is_false")]
    pub is_view: bool,
    /// Whether function can be used to initialize the state.
    #[serde(default, skip_serializing_if = "is_false")]
    pub is_init: bool,
    /// Whether function is accepting $NEAR.
    #[serde(default, skip_serializing_if = "is_false")]
    pub is_payable: bool,
    /// Whether function can only accept calls from self (current account).
    #[serde(default, skip_serializing_if = "is_false")]
    pub is_private: bool,
    /// Type identifiers of the function parameters.
    #[serde(default, skip_serializing_if = "AbiParameters::is_empty")]
    pub params: AbiParameters,
    /// Type identifiers of the callbacks of the function.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub callbacks: Vec<AbiType>,
    /// Type identifier of the vararg callbacks of the function.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub callbacks_vec: Option<AbiType>,
    /// Return type identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result: Option<AbiType>,
}

/// A list of function parameters sharing the same serialization type.
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, JsonSchema)]
#[serde(tag = "serialization_type")]
#[serde(rename_all = "lowercase")]
#[serde(deny_unknown_fields)]
pub enum AbiParameters {
    Json { args: Vec<AbiJsonParameter> },
    Borsh { args: Vec<AbiBorshParameter> },
}

impl Default for AbiParameters {
    fn default() -> Self {
        // JSON was picked arbitrarily for the default value, but generally it does not matter
        // whether this is JSON or Borsh.
        AbiParameters::Json { args: Vec::new() }
    }
}

impl AbiParameters {
    pub fn is_empty(&self) -> bool {
        match self {
            Self::Json { args } => args.is_empty(),
            Self::Borsh { args } => args.is_empty(),
        }
    }
}

/// Information about a single named JSON function parameter.
#[derive(Clone, Serialize, Deserialize, Debug, PartialEq, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct AbiJsonParameter {
    /// Parameter name (e.g. `p1` in `fn foo(p1: u32) {}`).
    pub name: String,
    /// JSON Subschema that represents this type (can be an inline primitive, a reference to the root schema and a few other corner-case things).
    pub type_schema: Schema,
}

/// Information about a single named Borsh function parameter.
#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct AbiBorshParameter {
    /// Parameter name (e.g. `p1` in `fn foo(p1: u32) {}`).
    pub name: String,
    /// Inline Borsh schema that represents this type.
    #[serde(with = "BorshSchemaContainerDef")]
    pub type_schema: BorshSchemaContainer,
}

impl JsonSchema for AbiBorshParameter {
    fn schema_name() -> String {
        "AbiBorshParameter".to_string()
    }

    fn json_schema(gen: &mut schemars::gen::SchemaGenerator) -> Schema {
        let mut name_schema_object = <String as JsonSchema>::json_schema(gen).into_object();
        name_schema_object.metadata().description =
            Some("Parameter name (e.g. `p1` in `fn foo(p1: u32) {}`).".to_string());

        let mut type_schema_object = Schema::Bool(true).into_object();
        type_schema_object.metadata().description =
            Some("Inline Borsh schema that represents this type.".to_string());

        let mut schema_object = schemars::schema::SchemaObject {
            instance_type: Some(schemars::schema::InstanceType::Object.into()),
            ..Default::default()
        };
        schema_object.metadata().description =
            Some("Information about a single named Borsh function parameter.".to_string());
        let object_validation = schema_object.object();
        object_validation
            .properties
            .insert("name".to_string(), name_schema_object.into());
        object_validation
            .properties
            // TODO: Narrow to BorshSchemaContainer once it derives JsonSchema
            .insert("type_schema".to_string(), type_schema_object.into());
        object_validation.required.insert("name".to_string());
        object_validation.required.insert("type_schema".to_string());
        object_validation.additional_properties =
            Some(schemars::schema::Schema::Bool(false).into());
        schema_object.into()
    }
}

// FIXME: Can be dropped once https://github.com/near/borsh-rs/pull/97 is released
impl Clone for AbiBorshParameter {
    fn clone(&self) -> Self {
        let type_schema = BorshSchemaContainer {
            declaration: self.type_schema.declaration.clone(),
            definitions: self
                .type_schema
                .definitions
                .iter()
                .map(|(k, v)| (k.clone(), borsh_clone::clone_definition(v)))
                .collect(),
        };
        Self {
            name: self.name.clone(),
            type_schema,
        }
    }
}

/// Information about a single type (e.g. return type).
#[derive(Serialize, Deserialize, Debug, PartialEq)]
#[serde(tag = "serialization_type")]
#[serde(rename_all = "lowercase")]
#[serde(deny_unknown_fields)]
pub enum AbiType {
    Json {
        /// JSON Subschema that represents this type (can be an inline primitive, a reference to the root schema and a few other corner-case things).
        type_schema: Schema,
    },
    Borsh {
        /// Inline Borsh schema that represents this type.
        #[serde(with = "BorshSchemaContainerDef")]
        type_schema: BorshSchemaContainer,
    },
}

impl JsonSchema for AbiType {
    fn schema_name() -> String {
        "AbiType".to_string()
    }

    fn json_schema(gen: &mut schemars::gen::SchemaGenerator) -> Schema {
        let mut json_abi_type = schemars::schema::SchemaObject::default();
        let json_abi_schema = json_abi_type.object();
        json_abi_schema
            .properties
            .insert("serialization_type".to_string(), {
                let schema = <String as JsonSchema>::json_schema(gen);
                let mut schema = schema.into_object();
                schema.enum_values = Some(vec!["json".into()]);
                schema.into()
            });
        json_abi_schema
            .properties
            .insert("type_schema".to_string(), gen.subschema_for::<Schema>());
        json_abi_schema
            .required
            .insert("serialization_type".to_string());
        json_abi_schema.required.insert("type_schema".to_string());
        json_abi_schema.additional_properties = Some(schemars::schema::Schema::Bool(false).into());

        let mut borsh_abi_type = schemars::schema::SchemaObject::default();
        let borsh_abi_schema = borsh_abi_type.object();
        borsh_abi_schema
            .properties
            .insert("serialization_type".to_string(), {
                let schema = <String as JsonSchema>::json_schema(gen);
                let mut schema = schema.into_object();
                schema.enum_values = Some(vec!["borsh".into()]);
                schema.into()
            });
        borsh_abi_schema
            .properties
            // TODO: Narrow to BorshSchemaContainer once it derives JsonSchema
            .insert(
                "type_schema".to_string(),
                schemars::schema::SchemaObject::default().into(),
            );
        borsh_abi_schema
            .required
            .insert("serialization_type".to_string());
        borsh_abi_schema.required.insert("type_schema".to_string());
        borsh_abi_schema.additional_properties = Some(schemars::schema::Schema::Bool(false).into());

        let mut schema_object = schemars::schema::SchemaObject {
            subschemas: Some(Box::new(schemars::schema::SubschemaValidation {
                one_of: Some(vec![
                    json_abi_type.into(),
                    borsh_abi_type.into(), // TODO: Narrow to BorshSchemaContainer once it derives JsonSchema
                ]),
                ..Default::default()
            })),
            ..Default::default()
        };
        schema_object.metadata().description =
            Some("Information about a single type (e.g. return type).".to_string());
        schema_object.into()
    }
}

// FIXME: Can be dropped once https://github.com/near/borsh-rs/pull/97 is released
impl Clone for AbiType {
    fn clone(&self) -> Self {
        match self {
            Self::Json { type_schema } => Self::Json {
                type_schema: type_schema.clone(),
            },
            Self::Borsh { type_schema } => {
                let type_schema = BorshSchemaContainer {
                    declaration: type_schema.declaration.clone(),
                    definitions: type_schema
                        .definitions
                        .iter()
                        .map(|(k, v)| (k.clone(), borsh_clone::clone_definition(v)))
                        .collect(),
                };
                Self::Borsh { type_schema }
            }
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(remote = "BorshSchemaContainer")]
struct BorshSchemaContainerDef {
    declaration: Declaration,
    #[serde(with = "borsh_serde")]
    definitions: HashMap<Declaration, Definition>,
}

/// Cloning functions for borsh types.
mod borsh_clone {
    use borsh::schema::{Definition, Fields};

    pub fn clone_fields(fields: &Fields) -> Fields {
        match fields {
            Fields::Empty => Fields::Empty,
            Fields::NamedFields(f) => Fields::NamedFields(f.clone()),
            Fields::UnnamedFields(f) => Fields::UnnamedFields(f.clone()),
        }
    }

    pub fn clone_definition(definition: &Definition) -> Definition {
        match definition {
            Definition::Array { length, elements } => Definition::Array {
                length: *length,
                elements: elements.clone(),
            },
            Definition::Sequence { elements } => Definition::Sequence {
                elements: elements.clone(),
            },
            Definition::Tuple { elements } => Definition::Tuple {
                elements: elements.clone(),
            },
            Definition::Enum { variants } => Definition::Enum {
                variants: variants.clone(),
            },
            Definition::Struct { fields } => Definition::Struct {
                fields: clone_fields(fields),
            },
        }
    }
}

/// This submodules follows <https://serde.rs/remote-derive.html> to derive Serialize/Deserialize for
/// `BorshSchemaContainer` parameters. The top-level serialization type is `HashMap<Declaration, Definition>`
/// for the sake of being easily plugged into `BorshSchemaContainerDef` (see its parameters).
mod borsh_serde {
    use super::*;
    use serde::ser::SerializeMap;
    use serde::{Deserializer, Serializer};

    #[derive(Serialize, Deserialize)]
    #[serde(remote = "Definition")]
    enum DefinitionDef {
        Array {
            length: u32,
            elements: Declaration,
        },
        #[serde(with = "transparent")]
        Sequence {
            elements: Declaration,
        },
        #[serde(with = "transparent")]
        Tuple {
            elements: Vec<Declaration>,
        },
        #[serde(with = "transparent")]
        Enum {
            variants: Vec<(VariantName, Declaration)>,
        },
        #[serde(with = "transparent_fields")]
        Struct {
            fields: Fields,
        },
    }

    #[derive(Serialize, Deserialize)]
    struct HelperDefinition(#[serde(with = "DefinitionDef")] Definition);

    /// #[serde(transparent)] does not support enum variants, so we have to use a custom ser/de impls for now.
    /// See <https://github.com/serde-rs/serde/issues/2092>.
    mod transparent {
        use serde::{Deserialize, Deserializer, Serialize, Serializer};

        pub fn serialize<T, S>(field: &T, serializer: S) -> Result<S::Ok, S::Error>
        where
            T: Serialize,
            S: Serializer,
        {
            serializer.serialize_some(&field)
        }

        pub fn deserialize<'de, T, D>(deserializer: D) -> Result<T, D::Error>
        where
            T: Deserialize<'de>,
            D: Deserializer<'de>,
        {
            T::deserialize(deserializer)
        }
    }

    /// Since `Fields` itself does not implement `Serialization`/`Deserialization`, we can't use
    /// `transparent` in combination with `#[serde(with = "...")]. Instead we have do it in this
    /// roundabout way.
    mod transparent_fields {
        use super::borsh_clone;
        use borsh::schema::{Declaration, FieldName, Fields};
        use serde::{Deserialize, Deserializer, Serialize, Serializer};

        #[derive(Serialize, Deserialize)]
        #[serde(remote = "Fields", untagged)]
        enum FieldsDef {
            NamedFields(Vec<(FieldName, Declaration)>),
            UnnamedFields(Vec<Declaration>),
            Empty,
        }

        #[derive(Serialize, Deserialize)]
        struct HelperFields(#[serde(with = "FieldsDef")] Fields);

        pub fn serialize<S>(fields: &Fields, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
        {
            HelperFields(borsh_clone::clone_fields(fields)).serialize(serializer)
        }

        pub fn deserialize<'de, D>(deserializer: D) -> Result<Fields, D::Error>
        where
            D: Deserializer<'de>,
        {
            Ok(HelperFields::deserialize(deserializer)?.0)
        }
    }

    pub fn serialize<S>(
        map: &HashMap<Declaration, Definition>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map_ser = serializer.serialize_map(Some(map.len()))?;
        for (k, v) in map {
            map_ser.serialize_entry(k, &HelperDefinition(borsh_clone::clone_definition(v)))?;
        }
        map_ser.end()
    }

    pub fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<HashMap<Declaration, Definition>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let map = HashMap::<Declaration, HelperDefinition>::deserialize(deserializer)?;
        Ok(map
            .into_iter()
            .map(|(k, HelperDefinition(v))| (k, v))
            .collect())
    }
}

fn is_false(b: &bool) -> bool {
    !b
}

#[cfg(test)]
mod tests {
    use super::*;
    use borsh::BorshSchema;
    use serde_json::Value;

    #[test]
    fn test_serde_array() {
        let abi_type = AbiType::Borsh {
            type_schema: <[u32; 2]>::schema_container(),
        };
        let value = serde_json::to_value(&abi_type).unwrap();
        let expected_json = r#"
          {
            "serialization_type": "borsh",
            "type_schema": {
              "declaration": "Array<u32, 2>",
              "definitions": {
                "Array<u32, 2>": {
                  "Array": {
                    "length": 2,
                    "elements": "u32"
                  }
                }
              }
            }
          }
        "#;
        let expected_value: Value = serde_json::from_str(expected_json).unwrap();
        assert_eq!(value, expected_value);

        if let AbiType::Borsh { type_schema } = serde_json::from_str(expected_json).unwrap() {
            assert_eq!(type_schema.declaration, "Array<u32, 2>".to_string());
            assert_eq!(type_schema.definitions.len(), 1);
            assert_eq!(
                type_schema.definitions.get("Array<u32, 2>").unwrap(),
                &Definition::Array {
                    length: 2,
                    elements: "u32".to_string()
                }
            );
        } else {
            panic!("Unexpected serialization type")
        }
    }

    #[test]
    fn test_serde_sequence() {
        let abi_type = AbiType::Borsh {
            type_schema: <Vec<u32>>::schema_container(),
        };
        let value = serde_json::to_value(&abi_type).unwrap();
        let expected_json = r#"
          {
            "serialization_type": "borsh",
            "type_schema": {
              "declaration": "Vec<u32>",
              "definitions": {
                "Vec<u32>": {
                  "Sequence": "u32"
                }
              }
            }
          }
        "#;
        let expected_value: Value = serde_json::from_str(expected_json).unwrap();
        assert_eq!(value, expected_value);

        if let AbiType::Borsh { type_schema } = serde_json::from_str(expected_json).unwrap() {
            assert_eq!(type_schema.declaration, "Vec<u32>".to_string());
            assert_eq!(type_schema.definitions.len(), 1);
            assert_eq!(
                type_schema.definitions.get("Vec<u32>").unwrap(),
                &Definition::Sequence {
                    elements: "u32".to_string()
                }
            );
        } else {
            panic!("Unexpected serialization type")
        }
    }

    #[test]
    fn test_serde_tuple() {
        let abi_type = AbiType::Borsh {
            type_schema: <(u32, u32)>::schema_container(),
        };
        let value = serde_json::to_value(&abi_type).unwrap();
        let expected_json = r#"
          {
            "serialization_type": "borsh",
            "type_schema": {
              "declaration": "Tuple<u32, u32>",
              "definitions": {
                "Tuple<u32, u32>": {
                  "Tuple": ["u32", "u32"]
                }
              }
            }
          }
        "#;
        let expected_value: Value = serde_json::from_str(expected_json).unwrap();
        assert_eq!(value, expected_value);

        if let AbiType::Borsh { type_schema } = serde_json::from_str(expected_json).unwrap() {
            assert_eq!(type_schema.declaration, "Tuple<u32, u32>".to_string());
            assert_eq!(type_schema.definitions.len(), 1);
            assert_eq!(
                type_schema.definitions.get("Tuple<u32, u32>").unwrap(),
                &Definition::Tuple {
                    elements: vec!["u32".to_string(), "u32".to_string()]
                }
            );
        } else {
            panic!("Unexpected serialization type")
        }
    }

    #[test]
    fn test_deser_enum() {
        #[derive(BorshSchema)]
        enum Either {
            _Left(u32),
            _Right(u32),
        }
        let abi_type = AbiType::Borsh {
            type_schema: <Either>::schema_container(),
        };
        let value = serde_json::to_value(&abi_type).unwrap();
        let expected_json = r#"
          {
            "serialization_type": "borsh",
            "type_schema": {
              "declaration": "Either",
              "definitions": {
                "Either": {
                  "Enum": [
                    ["_Left", "Either_Left"],
                    ["_Right", "Either_Right"]
                  ]
                },
                "Either_Left": {
                  "Struct": ["u32"]
                },
                "Either_Right": {
                  "Struct": ["u32"]
                }
              }
            }
          }
        "#;
        let expected_value: Value = serde_json::from_str(expected_json).unwrap();
        assert_eq!(value, expected_value);

        if let AbiType::Borsh { type_schema } = serde_json::from_str(expected_json).unwrap() {
            assert_eq!(type_schema.declaration, "Either".to_string());
            assert_eq!(type_schema.definitions.len(), 3);
            assert_eq!(
                type_schema.definitions.get("Either").unwrap(),
                &Definition::Enum {
                    variants: vec![
                        ("_Left".to_string(), "Either_Left".to_string()),
                        ("_Right".to_string(), "Either_Right".to_string())
                    ]
                }
            );
        } else {
            panic!("Unexpected serialization type")
        }
    }

    #[test]
    fn test_deser_struct_named() {
        #[derive(BorshSchema)]
        struct Pair {
            _first: u32,
            _second: u32,
        }
        let abi_type = AbiType::Borsh {
            type_schema: <Pair>::schema_container(),
        };
        let value = serde_json::to_value(&abi_type).unwrap();
        let expected_json = r#"
          {
            "serialization_type": "borsh",
            "type_schema": {
              "declaration": "Pair",
              "definitions": {
                "Pair": {
                  "Struct": [
                    ["_first", "u32"],
                    ["_second", "u32"]
                  ]
                }
              }
            }
          }
        "#;
        let expected_value: Value = serde_json::from_str(expected_json).unwrap();
        assert_eq!(value, expected_value);

        if let AbiType::Borsh { type_schema } = serde_json::from_str(expected_json).unwrap() {
            assert_eq!(type_schema.declaration, "Pair".to_string());
            assert_eq!(type_schema.definitions.len(), 1);
            assert_eq!(
                type_schema.definitions.get("Pair").unwrap(),
                &Definition::Struct {
                    fields: Fields::NamedFields(vec![
                        ("_first".to_string(), "u32".to_string()),
                        ("_second".to_string(), "u32".to_string())
                    ])
                }
            );
        } else {
            panic!("Unexpected serialization type")
        }
    }

    #[test]
    fn test_deser_struct_unnamed() {
        #[derive(BorshSchema)]
        struct Pair(u32, u32);
        let abi_type = AbiType::Borsh {
            type_schema: <Pair>::schema_container(),
        };
        let value = serde_json::to_value(&abi_type).unwrap();
        let expected_json = r#"
          {
            "serialization_type": "borsh",
            "type_schema": {
              "declaration": "Pair",
              "definitions": {
                "Pair": {
                  "Struct": [
                    "u32",
                    "u32"
                  ]
                }
              }
            }
          }
        "#;
        let expected_value: Value = serde_json::from_str(expected_json).unwrap();
        assert_eq!(value, expected_value);

        if let AbiType::Borsh { type_schema } = serde_json::from_str(expected_json).unwrap() {
            assert_eq!(type_schema.declaration, "Pair".to_string());
            assert_eq!(type_schema.definitions.len(), 1);
            assert_eq!(
                type_schema.definitions.get("Pair").unwrap(),
                &Definition::Struct {
                    fields: Fields::UnnamedFields(vec!["u32".to_string(), "u32".to_string()])
                }
            );
        } else {
            panic!("Unexpected serialization type")
        }
    }

    #[test]
    fn test_deser_struct_empty() {
        #[derive(BorshSchema)]
        struct Unit;
        let abi_type = AbiType::Borsh {
            type_schema: <Unit>::schema_container(),
        };
        let value = serde_json::to_value(&abi_type).unwrap();
        let expected_json = r#"
          {
            "serialization_type": "borsh",
            "type_schema": {
              "declaration": "Unit",
              "definitions": {
                "Unit": {
                  "Struct": null
                }
              }
            }
          }
        "#;
        let expected_value: Value = serde_json::from_str(expected_json).unwrap();
        assert_eq!(value, expected_value);

        if let AbiType::Borsh { type_schema } = serde_json::from_str(expected_json).unwrap() {
            assert_eq!(type_schema.declaration, "Unit".to_string());
            assert_eq!(type_schema.definitions.len(), 1);
            assert_eq!(
                type_schema.definitions.get("Unit").unwrap(),
                &Definition::Struct {
                    fields: Fields::Empty
                }
            );
        } else {
            panic!("Unexpected serialization type")
        }
    }

    #[test]
    fn test_deser_unknown_fields() {
        let json = r#"
          {
            "serialization_type": "borsh",
            "extra": "blah-blah",
            "type_schema": {
              "declaration": "Unit",
              "definitions": {
                "Unit": {
                  "Struct": null
                }
              }
            }
          }
        "#;
        serde_json::from_str::<AbiType>(json)
            .expect_err("Expected deserialization to fail due to unknown field");
    }

    #[test]
    fn test_deser_param() {
        #[derive(BorshSchema)]
        struct Unit;
        let expected_param = AbiBorshParameter {
            name: "foo".to_string(),
            type_schema: <Unit>::schema_container(),
        };
        let value = serde_json::to_value(&expected_param).unwrap();
        let expected_json = r#"
          {
            "name": "foo",
            "type_schema": {
              "declaration": "Unit",
              "definitions": {
                "Unit": {
                  "Struct": null
                }
              }
            }
          }
        "#;
        let expected_value: Value = serde_json::from_str(expected_json).unwrap();
        assert_eq!(value, expected_value);

        let param = serde_json::from_str::<AbiBorshParameter>(expected_json).unwrap();
        assert_eq!(param.name, "foo");
        assert_eq!(param.type_schema.declaration, "Unit".to_string());
        assert_eq!(param.type_schema.definitions.len(), 1);
        assert_eq!(
            param.type_schema.definitions.get("Unit").unwrap(),
            &Definition::Struct {
                fields: Fields::Empty
            }
        );
    }

    #[test]
    fn test_deser_param_unknown_fields() {
        let json = r#"
          {
            "name": "foo",
            "extra": "blah-blah",
            "type_schema": {
              "declaration": "Unit",
              "definitions": {
                "Unit": {
                  "Struct": null
                }
              }
            }
          }
        "#;
        serde_json::from_str::<AbiBorshParameter>(json)
            .expect_err("Expected deserialization to fail due to unknown field");
    }

    #[test]
    fn test_correct_version() {
        let json = format!(
            r#"
            {{
                "schema_version": "{}",
                "metadata": {{}},
                "body": {{
                    "functions": [],
                    "root_schema": {{}}
                }}
            }}
            "#,
            SCHEMA_VERSION
        );
        let abi_root = serde_json::from_str::<AbiRoot>(&json).unwrap();
        assert_eq!(abi_root.schema_version, SCHEMA_VERSION);
    }

    #[test]
    fn test_older_version() {
        let json = r#"
          {
            "schema_version": "0.0.1",
            "metadata": {},
            "body": {
                "functions": [],
                "root_schema": {}
            }
          }
        "#;
        let err = serde_json::from_str::<AbiRoot>(json)
            .expect_err("Expected deserialization to fail due to schema version mismatch");
        assert!(err.to_string().contains(
            "got 0.0.1: consider re-generating your ABI file with a newer version of SDK and cargo-near"
        ));
    }

    #[test]
    fn test_newer_version() {
        let json = r#"
          {
            "schema_version": "99.99.99",
            "metadata": {},
            "body": {
                "functions": [],
                "root_schema": {}
            }
          }
        "#;
        let err = serde_json::from_str::<AbiRoot>(json)
            .expect_err("Expected deserialization to fail due to schema version mismatch");
        assert!(err
            .to_string()
            .contains("got 99.99.99: consider upgrading near-abi to a newer version"));
    }
}
