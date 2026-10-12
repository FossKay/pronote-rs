use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("invalid credentials")]
    InvalidCredentials,

    #[error("invalid PRONOTE start page")]
    InvalidStartPage,

    #[error("PRONOTE session expired")]
    Expired,

    #[error("unsupported client type")]
    UnsupportedClient,

    #[error("access to tab {0} not authorized")]
    UnauthorizedTab(u64),

    #[error("invalid action blocked")]
    InvalidAction,

    #[error("server unreachable")]
    ConnectionFailed,

    #[error("PRONOTE error {code}: {details}")]
    Pronote { code: i64, details: String },

    #[error("invalid server response")]
    InvalidResponse,
}
