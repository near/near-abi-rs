use near_abi::AbiRoot;

fn main() -> anyhow::Result<()> {
    let mut gen = schemars::gen::SchemaGenerator::default();
    let schema = gen.root_schema_for::<AbiRoot>();
    println!("{}", serde_json::to_string_pretty(&schema)?);
    Ok(())
}
