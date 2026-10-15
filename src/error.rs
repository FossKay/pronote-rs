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
    InvalidResponse,
}
