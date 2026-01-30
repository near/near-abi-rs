use near_abi::AbiRoot;

fn main() -> anyhow::Result<()> {
    let mut schema_gen = schemars::r#gen::SchemaGenerator::default();
    let schema = schema_gen.root_schema_for::<AbiRoot>();
    println!("{}", serde_json::to_string_pretty(&schema)?);
    Ok(())
}
