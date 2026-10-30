mod client;
mod communication;
mod data;
mod encryption;
mod helpers;

pub use client::{Client, parse_key};
pub use data::*;
pub use encryption::Encryption;

#[derive(Debug)]
pub enum Error {
    InvalidCredentials,
    InvalidStartPage,
    Expired,
    UnsupportedClient,
    UnauthorizedTab(u64),
    InvalidAction,
    ConnectionFailed,
    Pronote { code: i64, details: String },
}
