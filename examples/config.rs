use anyhow::Result;
use rust_xdiff::DiffConfig;

fn main() -> Result<()> {
    let content = include_str!("../fixures/test.yml");
    let config = DiffConfig::from_yaml(content)?;

    println!("{:#?}", config);

    Ok(())
}
