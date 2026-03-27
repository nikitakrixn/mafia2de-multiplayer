//! Типы ошибок лаунчера.

use std::fmt;

#[derive(Debug)]
pub enum Error {
    GameAlreadyRunning,
    SteamNotRunning,
    GamePathNotFound,
    DllNotFound(String),
    DllInvalid(String),
    InjectionFailed(String),
    ProcessFailed(String),
    Registry(String),
    Windows(windows::core::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::GameAlreadyRunning => write!(f, "Game is already running"),
            Self::SteamNotRunning => write!(f, "Steam is not running. Please start Steam first."),
            Self::GamePathNotFound => write!(f, "Game executable not found"),
            Self::DllNotFound(p) => write!(f, "Client DLL not found: {p}"),
            Self::DllInvalid(msg) => write!(f, "Invalid DLL: {msg}"),
            Self::InjectionFailed(msg) => write!(f, "DLL injection failed: {msg}"),
            Self::ProcessFailed(msg) => write!(f, "Process error: {msg}"),
            Self::Registry(msg) => write!(f, "Registry error: {msg}"),
            Self::Windows(e) => write!(f, "Windows error: {e}"),
        }
    }
}

impl std::error::Error for Error {}

impl From<windows::core::Error> for Error {
    fn from(e: windows::core::Error) -> Self {
        Self::Windows(e)
    }
}
