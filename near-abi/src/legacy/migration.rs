use super::v0_1;
use super::v0_2;

pub trait ToBorshSchema {
    fn to_borsh_schema(self) -> borsh::schema::BorshSchemaContainer;
}

impl ToBorshSchema for v0_1::AbiType {
    fn to_borsh_schema(self) -> borsh::schema::BorshSchemaContainer {
        if let v0_1::AbiType::Borsh { type_schema } = self {
            type_schema
        } else {
            panic!("Expected Borsh serialization type, but got {:?}", self)
        }
    }
}

pub trait ToJsonSchema {
    fn to_json_schema(self) -> schemars::schema::Schema;
}

impl ToJsonSchema for v0_1::AbiType {
    fn to_json_schema(self) -> schemars::schema::Schema {
        if let v0_1::AbiType::Json { type_schema } = self {
            type_schema
        } else {
            panic!("Expected Borsh serialization type, but got {:?}", self)
        }
    }
}

fn v0_1_abi_type_to_v0_2(abi_type: v0_1::AbiType) -> v0_2::AbiType {
    match abi_type {
        v0_1::AbiType::Json { type_schema } => v0_2::AbiType::Json { type_schema },
        v0_1::AbiType::Borsh { type_schema } => v0_2::AbiType::Borsh { type_schema },
    }
}

fn v0_2_abi_type_to_current(abi_type: v0_2::AbiType) -> crate::AbiType {
    match abi_type {
        v0_2::AbiType::Json { type_schema } => crate::AbiType::Json { type_schema },
        v0_2::AbiType::Borsh { type_schema } => crate::AbiType::Borsh { type_schema },
    }
}

pub(crate) fn v0_1_to_v0_2(abi: v0_1::AbiRoot) -> v0_2::AbiRoot {
    // Should be safe to unwrap as metadata is supposed to be always compatible between versions
    let metadata: v0_2::AbiMetadata =
        serde_json::from_value(serde_json::to_value(&abi.metadata).unwrap()).unwrap();
    v0_2::AbiRoot {
        schema_version: v0_2::SCHEMA_VERSION.to_string(),
        metadata,
        body: v0_2::AbiBody {
            functions: abi
                .body
                .functions
                .into_iter()
                .map(|f| {
                    let is_json = f
                        .params
                        .first()
                        .map(|p| matches!(p.typ, v0_1::AbiType::Json { .. }))
                        .unwrap_or(true);
                    let params = if is_json {
                        v0_2::AbiParameters::Json {
                            args: f
                                .params
                                .into_iter()
                                .map(|p| v0_2::AbiJsonParameter {
                                    name: p.name,
                                    type_schema: p.typ.to_json_schema(),
                                })
                                .collect(),
                        }
                    } else {
                        v0_2::AbiParameters::Borsh {
                            args: f
                                .params
                                .into_iter()
                                .map(|p| v0_2::AbiBorshParameter {
                                    name: p.name,
                                    type_schema: p.typ.to_borsh_schema(),
                                })
                                .collect(),
                        }
                    };
                    v0_2::AbiFunction {
                        name: f.name,
                        doc: f.doc,
                        is_view: f.is_view,
                        is_init: f.is_init,
                        is_payable: f.is_payable,
                        is_private: f.is_private,
                        params,
                        callbacks: f.callbacks.into_iter().map(v0_1_abi_type_to_v0_2).collect(),
                        callbacks_vec: f.callbacks_vec.map(v0_1_abi_type_to_v0_2),
                        result: f.result.map(v0_1_abi_type_to_v0_2),
                    }
                })
                .collect(),
            root_schema: abi.body.root_schema,
        },
    }
}

pub(crate) fn v0_2_to_current(abi: v0_2::AbiRoot) -> crate::AbiRoot {
    // Should be safe to unwrap as metadata is supposed to be always compatible between versions
    let metadata: crate::AbiMetadata =
        serde_json::from_value(serde_json::to_value(&abi.metadata).unwrap()).unwrap();
    crate::AbiRoot {
        schema_version: crate::SCHEMA_VERSION.to_string(),
        metadata,
        body: crate::AbiBody {
            functions: abi
                .body
                .functions
                .into_iter()
                .map(|f| crate::AbiFunction {
                    name: f.name,
                    doc: f.doc,
                    is_view: f.is_view,
                    is_init: f.is_init,
                    is_payable: f.is_payable,
                    is_private: f.is_private,
                    params: match f.params {
                        v0_2::AbiParameters::Json { args } => crate::AbiParameters::Json {
                            args: args
                                .into_iter()
                                .map(|a| crate::AbiJsonParameter {
                                    name: a.name,
                                    type_schema: a.type_schema,
                                })
                                .collect(),
                        },
                        v0_2::AbiParameters::Borsh { args } => crate::AbiParameters::Borsh {
                            args: args
                                .into_iter()
                                .map(|a| crate::AbiBorshParameter {
                                    name: a.name,
                                    type_schema: a.type_schema,
                                })
                                .collect(),
                        },
                    },
                    callbacks: f
                        .callbacks
                        .into_iter()
                        .map(v0_2_abi_type_to_current)
                        .collect(),
                    callbacks_vec: f.callbacks_vec.map(v0_2_abi_type_to_current),
                    result: f.result.map(v0_2_abi_type_to_current),
                })
                .collect(),
            root_schema: abi.body.root_schema,
        },
    }
}
