use crate::ExtraArgs;
use anyhow::{anyhow, Result};
use clap::{Parser, Subcommand};

/// Diff two http requests and compares
#[derive(Parser, Debug, Clone)]
#[clap(version, author, about, long_about=None)]
pub struct Args {
    #[clap(subcommand)]
    pub action: Action,
}

#[derive(Subcommand, Debug, Clone)]
#[non_exhaustive]
pub enum Action {
    ///Diff two API responses based on given profile
    Run(RunArgs),
    /// Parse URLs to generate a profile
    Parse,
}

#[derive(Parser, Debug, Clone)]
pub struct RunArgs {
    /// Configuration to use.
    #[clap(short, long, value_parser)]
    pub config: Option<String>,

    ///Profile name
    #[clap(short, long, value_parser)]
    pub profile: String,

    /// Override args.
    /// For query params, use `-e key=value`.
    /// For headers, use `-e %key=value`.
    /// For body, use `-e @key=value`.
    #[clap(short, long, value_parser=parse_key_val, number_of_values=1)]
    pub extra_params: Vec<KeyVal>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyValType {
    Query,
    Header,
    Body,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyVal {
    key_type: KeyValType,
    key: String,
    val: String,
}

fn parse_key_val(s: &str) -> Result<KeyVal> {
    let mut parts = s.splitn(2, '=');
    let key = parts
        .next()
        .ok_or(anyhow!("Invalid key value pair: {}", s))?
        .trim();
    let value = parts
        .next()
        .ok_or(anyhow!("Invalid key value pair: {}", s))?
        .trim();

    let (key_type, key) = match key.chars().next() {
        Some('%') => (KeyValType::Header, &key[1..]),
        Some('@') => (KeyValType::Body, &key[1..]),
        Some(v) if v.is_ascii_alphabetic() => (KeyValType::Query, key),
        _ => return Err(anyhow!("Invalid key value pair")),
    };

    Ok(KeyVal {
        key_type,
        key: key.to_string(),
        val: value.to_string(),
    })
}

impl From<Vec<KeyVal>> for ExtraArgs {
    fn from(args: Vec<KeyVal>) -> Self {
        let mut headers = vec![];
        let mut query = vec![];
        let mut body = vec![];

        for arg in args {
            match arg.key_type {
                KeyValType::Header => headers.push((arg.key, arg.val)),
                KeyValType::Query => query.push((arg.key, arg.val)),
                KeyValType::Body => body.push((arg.key, arg.val)),
            }
        }

        Self {
            headers,
            query,
            body,
        }
    }
}
