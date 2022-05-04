use std::ffi::OsString;

/// Mount remote file over HTTP
#[derive(argh::FromArgs)]
pub struct Args {
    /// file name (default "file")
    #[argh(option, short = 'f', default = "OsString::from(\"file\")")]
    pub filename: OsString,

    /// cache size (default 10MiB)
    #[argh(option, short = 's', default = "10 * 1024 * 1024", from_str_fn(parse_size))]
    pub cache_size: usize,

    /// path to an empty directory
    #[argh(positional)]
    pub mountpoint: OsString,

    /// URL pointing to a file to mount
    #[argh(positional)]
    pub url: String,
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
