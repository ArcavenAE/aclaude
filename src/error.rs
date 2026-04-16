use thiserror::Error;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ForestageError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),

    #[error("config error: {message}")]
    Config { message: String },

    #[error("toml parse error in {path}: {source}")]
    Toml {
        path: String,
        source: toml::de::Error,
    },

    #[error("yaml parse error in {path}: {source}")]
    Yaml {
        path: String,
        source: serde_yaml::Error,
    },

    #[error("theme not found: {slug}")]
    ThemeNotFound { slug: String },

    #[error("character not found: {character} in theme {theme}")]
    CharacterNotFound { character: String, theme: String },

    #[error("session error: {message}")]
    Session { message: String },

    #[error("claude CLI not found — install Claude Code first")]
    ClaudeNotFound,

    #[error("update error: {message}")]
    Update { message: String },

    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, ForestageError>;
