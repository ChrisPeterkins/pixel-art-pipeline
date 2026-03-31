use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum PipelineError {
    #[error("Config error: {0}")]
    Config(String),

    #[error("Failed to parse config file {path}: {source}")]
    ConfigParse {
        path: PathBuf,
        source: serde_yaml_ng::Error,
    },

    #[error("File not found: {0}")]
    FileNotFound(PathBuf),

    #[error("No files matched pattern: {0}")]
    NoFilesMatched(String),

    #[error("Image error for {path}: {source}")]
    Image {
        path: PathBuf,
        source: image::ImageError,
    },

    #[error("Palette error: {0}")]
    Palette(String),

    #[error("Packing error: {0}")]
    Packing(String),

    #[error("Animation error: {0}")]
    Animation(String),

    #[error("Output error: {0}")]
    Output(String),

    #[error("IO error for {path}: {source}")]
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
}

pub type Result<T> = std::result::Result<T, PipelineError>;
