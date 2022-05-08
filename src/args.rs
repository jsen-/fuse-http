use clap::{AppSettings, Parser};
use std::ffi::OsString;

/// Mount remote file over HTTP
#[derive(Parser)]
#[clap(version, setting = AppSettings::DeriveDisplayOrder)]
pub struct Args {
    /// file name
    #[clap(long, short = 'f', default_value = "\"file\"", display_order = 1)]
    pub filename: OsString,

    /// cache size
    #[clap(long, short = 's', default_value = "10MiB", parse(try_from_str = parse_size))]
    pub cache_size: usize,

    /// keep the process running in foreground
    #[clap(long)]
    pub no_daemonize: bool,

    /// path to an empty directory
    pub mountpoint: OsString,

    /// URL pointing to a file to mount
    pub url: String,
}

pub fn args() -> Args {
    Args::parse()
}

fn parse_size(input: &str) -> Result<usize, String> {
    let b = byte_unit::Byte::from_str(input)
        .map_err(|err| format!("Could not convert unit to bytes: {err}"))
        .map(|byte| byte.get_bytes())?;
    if b > usize::MAX as u64 {
        return Err(format!("cache size must be less than or equal to {}", usize::MAX));
    }
    if b < 1 {
        return Err("cache size must be a positive integer".to_string());
    }
    #[allow(clippy::cast_possible_truncation)] // we checked all invariants above
    Ok(b as usize)
}
