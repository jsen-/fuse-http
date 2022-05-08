use std::io;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("Request error: {0}")]
    Req(#[from] ureq::Error),

    #[error(r#"Missing "content-length" header"#)]
    UnknownLength,

    #[error(r#"Missing or unknown "accept-ranges" header"#)]
    MissingOrUnknownAcceptRanges,

    #[error("Unexpected status: {0}")]
    UnexpectedStatus(u16),

    #[error(r#"Could not parse "content-length" header: {0}"#)]
    ParseLength(String),

    #[error("Mount error: {0}")]
    Mount(#[from] io::Error),

    #[error("Daemonize error: {0}")]
    Daemonize(#[from] daemonize::DaemonizeError),
}
