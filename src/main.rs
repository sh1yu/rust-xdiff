use anyhow::{anyhow, Result};
use clap::Parser;
use dialoguer::theme::ColorfulTheme;
use dialoguer::{Input, MultiSelect};
use rust_xdiff::cli::{Action, Args, RunArgs};
use rust_xdiff::{DiffConfig, DiffProfile, RequestProfile, ResponseProfile};
use std::io::Write;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();
    match args.action {
        Action::Run(run_args) => run(run_args).await?,
        Action::Parse => parse()?,
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

    let output = profile.diff(args.extra_params.into()).await?;
    let stdout = std::io::stdout();
    let mut stdout = stdout.lock();
    write!(stdout, "{}", output)?;
    Ok(())
}

fn parse() -> Result<()> {
    let theme = &ColorfulTheme::default();
    let url1: String = Input::with_theme(theme)
        .with_prompt("url1")
        .interact_text()?;
    let url2: String = Input::with_theme(theme)
        .with_prompt("url2")
        .interact_text()?;
    let profile: String = Input::with_theme(theme)
        .with_prompt("profile")
        .interact_text()?;
    let skip_candidate_headers = ["report-to", "date", "age"];
    let chosen = MultiSelect::with_theme(theme)
        .with_prompt("select headers to skip")
        .items(&skip_candidate_headers)
        .interact()?;
    let skip_headers = chosen
        .iter()
        .map(|i| skip_candidate_headers[*i].to_string())
        .collect();

    let req1: RequestProfile = url1.parse()?;
    let req2: RequestProfile = url2.parse()?;
    let res = ResponseProfile::new(skip_headers, vec![]);
    let profile = DiffProfile::new(req1, req2, res);
    let result = serde_yaml::to_string(&profile)?;

    let stdout = std::io::stdout();
    let mut stdout = stdout.lock();

    Ok(())
}
