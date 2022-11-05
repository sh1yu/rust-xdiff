use anyhow::{anyhow, Result};
use clap::Parser;
use rust_xdiff::cli::{Action, Args, RunArgs};
use rust_xdiff::DiffConfig;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    match args.action {
        Action::Run(run_args) => run(run_args).await?,
        _ => panic!("not implemented"),
    }

    Ok(())
}

async fn run(args: RunArgs) -> Result<()> {
    let config_file = args.config.unwrap_or("./xdiff.yml".to_string());
    let config = DiffConfig::load_yaml(&config_file).await?;
    let profile = config.get_profile(&args.profile).ok_or(anyhow!(
        "Profile {} not found in config file {}",
        args.profile,
        config_file
    ))?;

    profile.diff(args.extra_params.into()).await?;
    Ok(())
}
