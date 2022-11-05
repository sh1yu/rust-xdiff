pub mod cli;
mod config;
mod req;
mod utils;

pub use config::{DiffConfig, DiffProfile, ResponseProfile};
pub use req::RequestProfile;
pub use utils::diff_text;

#[derive(Debug)]
pub struct ExtraArgs {
    headers: Vec<(String, String)>,
    query: Vec<(String, String)>,
    body: Vec<(String, String)>,
}
