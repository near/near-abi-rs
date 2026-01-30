use near_abi::AbiRoot;
use schemars::SchemaGenerator;

fn main() -> anyhow::Result<()> {
    let schema_gen = SchemaGenerator::default();
    let schema = schema_gen.into_root_schema_for::<AbiRoot>();
    println!("{}", serde_json::to_string_pretty(&schema)?);
    Ok(())
}
