#![warn(clippy::pedantic)]

mod args;
mod error;

use args::Args;
use error::Error;

use fuser::{FileAttr, FileType, Filesystem, MountOption, ReplyAttr, ReplyEntry, Request, Session};
use libc::{EIO, ENOENT, ENOSPC, ERANGE};
use std::{
    ffi::{OsStr, OsString},
    io::Read,
    path::Path,
    process, str,
    time::{Duration, UNIX_EPOCH},
};

const TTL: Duration = Duration::from_secs(1); // 1 second
const BLKSIZE: u32 = 4096;

const ROOT_DIR_ATTR: FileAttr = FileAttr {
    ino: 1,
    size: 0,
    blocks: 0,
    atime: UNIX_EPOCH, // 1970-01-01 00:00:00
    mtime: UNIX_EPOCH,
    ctime: UNIX_EPOCH,
    crtime: UNIX_EPOCH,
    kind: FileType::Directory,
    perm: 0o444,
    nlink: 0,
    uid: 0,
    gid: 0,
    rdev: 0,
    flags: 0,
    blksize: 512,
};

struct HttpFs {
    file_attr: FileAttr,
    url: String,
    filename: OsString,
    cache_size: usize,
    cache: Vec<u8>,
    cache_pos: Option<(u64, usize)>,
}

impl Filesystem for HttpFs {
    fn lookup(&mut self, _req: &Request, parent: u64, name: &OsStr, reply: ReplyEntry) {
        log::info!("lookup parent:{parent} name:{}", Path::new(name).display());
        if parent == 1 && name == self.filename {
            reply.entry(&TTL, &self.file_attr, 0);
        } else {
            reply.error(ENOENT);
        }
    }
    fn getattr(&mut self, _req: &Request, ino: u64, reply: ReplyAttr) {
        log::info!("getattr ino:{ino}");
        match ino {
            1 => reply.attr(&TTL, &ROOT_DIR_ATTR),
            2 => reply.attr(&TTL, &self.file_attr),
            _ => reply.error(ENOENT),
        }
    }
    fn readdir(&mut self, _req: &Request<'_>, ino: u64, _fh: u64, offset: i64, mut reply: fuser::ReplyDirectory) {
        log::info!("readdir ino:{ino} offset:{offset}");
        if ino == 1 {
            if offset == 0 {
                let _ = reply.add(1, 0, FileType::Directory, &Path::new("."));
                let _ = reply.add(1, 1, FileType::Directory, &Path::new(".."));
                let _ = reply.add(1, 1, FileType::RegularFile, &Path::new(&self.filename));
            }
            reply.ok();
        } else {
            reply.error(ENOENT);
        }
    }
    fn read(
        &mut self,
        _req: &Request<'_>,
        ino: u64,
        _fh: u64,
        offset: i64,
        size: u32,
        _flags: i32,
        _lock_owner: Option<u64>,
        reply: fuser::ReplyData,
    ) {
        log::info!("read ino:{ino} offset:{offset} size:{size}");
        if ino != 2 {
            return reply.error(ENOENT);
        }
        if offset < 0 {
            return reply.error(ERANGE);
        }
        if size == 0 {
            return reply.data(&[]);
        }
        if size as usize > self.cache_size {
            log::error!("cache size ({}) too small for requested chunk size: {}", self.cache_size, size);
            return reply.error(ENOSPC);
        }
        let len = size as usize;
        #[allow(clippy::cast_sign_loss)] // we checked this above
        let start = offset as u64;

        if let Some((cache_start, cache_len)) = self.cache_pos {
            if start >= cache_start {
                let cache_offset = (start - cache_start) as usize;
                if cache_offset + len <= cache_len {
                    return reply.data(&self.cache[cache_offset..cache_offset + len]);
                }
            }
        }

        let data = match self.req(start, len) {
            Ok(resp) => resp,
            Err(err) => {
                eprintln!("{err}");
                return reply.error(EIO);
            }
        };
        reply.data(data);
    }
}

impl HttpFs {
    fn req(&mut self, start: u64, size: usize) -> Result<&[u8], Error> {
        let end = start + self.cache_size as u64 - 1;
        log::info!("req range:{start}-{end}");
        let resp = ureq::get(&self.url).set("Range", &format!("bytes={}-{}", start, end)).call()?;
        log::trace!("{resp:?} {:?}", resp.headers_names());
        let mut buf = Vec::with_capacity(self.cache_size as usize);
        let nbytes = resp.into_reader().take(self.cache_size as u64).read_to_end(&mut buf)?;
        // TODO: make sure nbytes == resp.headers["content-length"]
        self.cache = buf;
        self.cache_pos = Some((start, nbytes));
        Ok(&self.cache[..size])
    }
}

fn real_main(args: Args) -> Result<(), Error> {
    let resp = ureq::head(&args.url).call()?;
    log::trace!("{resp:?} {:?}", resp.headers_names());
    if resp.status() != 200 {
        return Err(Error::UnexpectedStatus(resp.status()));
    }
    if resp.header("accept-ranges") != Some("bytes") {
        return Err(Error::MissingOrUnknownAcceptRanges);
    }
    let content_len = resp.header("content-length").ok_or(Error::UnknownLength)?;
    let size = str::parse::<u64>(content_len).map_err(|_| Error::ParseLength(content_len.to_string()))?;

    let fs = HttpFs {
        filename: args.filename,
        url: args.url.clone(),
        cache_size: args.cache_size,
        file_attr: FileAttr {
            ino: 2,
            size,
            blocks: if size == 0 { 0 } else { (size - 1) / u64::from(BLKSIZE) + 1 },
            atime: UNIX_EPOCH,
            mtime: UNIX_EPOCH,
            ctime: UNIX_EPOCH,
            crtime: UNIX_EPOCH,
            kind: FileType::RegularFile,
            perm: 0o444,
            nlink: 0,
            uid: 0,
            gid: 0,
            rdev: 0,
            flags: 0,
            blksize: BLKSIZE,
        },
        cache: Vec::new(),
        cache_pos: None,
    };

    let mut session = Session::new(
        fs,
        &args.mountpoint.as_ref(),
        &[MountOption::RO, MountOption::FSName("fuse-http".into()), MountOption::Subtype(args.url)],
    )?;
    if !args.no_daemonize {
        daemonize::Daemonize::new().start()?;
    }
    session.run()?;
    Ok(())
}

fn main() {
    env_logger::init();

    let args: Args = argh::from_env();
    if let Err(err) = real_main(args) {
        eprintln!("{err}");
        process::exit(1);
    }
}
