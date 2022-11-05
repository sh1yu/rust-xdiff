pub mod cli;
mod config;

pub use config::{DiffConfig, DiffProfile, RequestProfile, ResponseProfile};

#[derive(Debug)]
pub struct ExtraArgs {
    headers: Vec<(String, String)>,
    query: Vec<(String, String)>,
    body: Vec<(String, String)>,
}
