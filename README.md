# fuse-http

Read-only mount URL pointing to a file into local directory using [FUSE](https://www.kernel.org/doc/html/latest/filesystems/fuse.html)  
The remote server must support ranged requests (respond with `Accept-Ranges: bytes` header to `HEAD` request to the provided URL) and respond with `206 Partial Content` when `Range: bytes=<start>-<end>` request header is present with `GET` request.


## installation

```
cargo build --release
```

For maximum portability you can use [build-musl.sh](build-musl.sh). This will build the project with [MUSL](https://musl.libc.org) which requires no runtime libraries whatsoever, but requires nightly rust and musl-toolchain installed on the build host.
```
$ ldd target/x86_64-unknown-linux-musl/release/fuse-http
        statically linked
```


## usage

<pre>
$ fuse-http --help
<span style='color: #0dbc79;'>fuse-http</span> 0.3.0
Mount remote file over HTTP

<span style='color: #999900;'>USAGE:</span>
fuse-http [OPTIONS] &lt;MOUNTPOINT&gt; &lt;URL&gt;

<span style='color: #999900;'>ARGS:</span>
    <span style='color: #0dbc79;'>&lt;MOUNTPOINT&gt;</span>    path to an empty directory
    <span style='color: #0dbc79;'>&lt;URL&gt;</span>           URL pointing to a file to mount

<span style='color: #999900;'>OPTIONS:</span>
    <span style='color: #0dbc79;'>-f</span>, <span style='color: #0dbc79;'>--filename &lt;FILENAME&gt;</span>        file name [default: "file"]
    <span style='color: #0dbc79;'>-s</span>, <span style='color: #0dbc79;'>--cache-size &lt;CACHE_SIZE&gt;</span>    cache size [default: 10MiB]
    <span style='color: #0dbc79;'>--no-daemonize</span>                   keep the process running in foreground
    <span style='color: #0dbc79;'>-h</span>, <span style='color: #0dbc79;'>--help</span>                       Print help information
    <span style='color: #0dbc79;'>-V</span>, <span style='color: #0dbc79;'>--version</span>                    Print version information
</pre>

example:
```sh
$ fuse-http \
    --cache-size 50MiB \
    --filename disk_img \
    /tmp/remote_image \
    http://192.168.0.37/files/ubuntu.iso

$ ls -la /tmp/remote_image/
total 2097152
dr--r--r--  0 root root          0 Jan  1  1970 .
drwxrwxrwt 19 root root        600 May  4 13:14 ..
-r--r--r--  0 root root 2147483648 Jan  1  1970 disk_img

$ time md5sum /tmp/remote_image/disk_img
f44196d7c5ce6a05770206b165c4b414  /tmp/remote_image/disk_img

real    0m7,041s
user    0m3,157s
sys     0m0,558s

$ time qemu-img convert /tmp/remote_image/disk_img -O raw /tmp/tmp.img

real    0m11,112s
user    0m0,147s
sys     0m1,579s

$ umount /tmp/remote_image
```

Log verbosity is controlled by environment variable [RUST_LOG](https://docs.rs/env_logger/latest/env_logger/#enabling-logging)
```sh
RUST_LOG=fuse_http=trace fuse-http ...
```


## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.


## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
