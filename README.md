# rust-g

rust-g (pronounced rusty-g) is a library which offloads certain expensive or
difficult tasks from BYOND.

This library is currently used in the [/tg/station] codebase, and is required for
it to run. A pre-compiled DLL version can be found in the repo root of codebases that use it,
but you can build your own from this repo (and you should if you're running a server).

Builds can also be found on the [releases page] **but should only be used for Windows**,
as Linux has compatibility issues across distributions.

[releases page]: https://github.com/tgstation/rust-g/releases

## Dependencies

The [Rust] compiler:

1. Install the Rust compiler's dependencies (primarily the system linker):

   * Ubuntu: `sudo apt-get install gcc-multilib`
   * Windows (MSVC): [Build Tools for Visual Studio 2017][msvc]
   * Windows (GNU): No action required

1. Use [the Rust installer](https://rustup.rs/), or another Rust installation method,
   or run the following:

    ```sh
    curl https://sh.rustup.rs -sSfo rustup-init.sh
    chmod +x rustup-init.sh
    ./rustup-init.sh
    ```

1. Set the default compiler to **32-bit**:

    ```sh
    # Clone the `rust-g` repository to a directory of your choice
    git clone https://github.com/tgstation/rust-g.git
    # in the `rust-g` directory...
    cd rust-g
    # Linux
    rustup target add i686-unknown-linux-gnu
    # Windows
    rustup target add i686-pc-windows-msvc
    ```

System libraries:

* Ubuntu and Debian users run:

    ```sh
    sudo dpkg --add-architecture i386
    sudo apt-get update
    sudo apt-get install zlib1g-dev:i386
    ```

* Other Linux distributions install the appropriate **32-bit development** and **32-bit runtime** packages.

If you want to use the `pc-windows-gnu` or similar other target ABI, do the following:
1. Change the `"rust-analyzer.cargo.target"` setting in `.cargo/config` to `i686-pc-windows-gnu`.
2. Run `git update-index --assume-unchanged .cargo/config`, which will tell git to 'ignore' the changes you made.
3. If you find yourself ever wanting to change back, run `git update-index --no-assume-unchanged .cargo/config`.

## Compiling

The [Cargo] tool handles compilation, as well as automatically downloading and
compiling all Rust dependencies. The default configuration is suitable for
use with the [/tg/station] codebase. To compile in release mode (recommended for
speed):

Linux:
```sh
export PKG_CONFIG_ALLOW_CROSS=1
cargo build --release --target i686-unknown-linux-gnu
# output: target/i686-unknown-linux-gnu/release/librust_g.so
```

Windows:

```sh
cargo build --release --target i686-pc-windows-msvc
# output: target/i686-pc-windows-msvc/release/rust_g.dll
```

If you aren't sharing the binary with other people, consider compiling [targeting your native cpu](https://rust-lang.github.io/packed_simd/perf-guide/target-feature/rustflags.html#target-cpu) for potential performance improvements. You can do this by setting the `RUSTFLAGS` environment variable to `-C target-cpu=native`. For example, in Powershell you would use `$Env:RUSTFLAGS="-C target-cpu=native"`.

To get additional features, pass a list to `--features`, for example `--features hash,url`. To get all features, pass `--features all`. To disable the default features, pass `--no-default-features`.
You can't use `--all-features` because of conflicting `native_tls` and `rustls_tls` features to select the mysql backend.

The default features are:
* acreplace: Aho-Corasick string matching and replacement.
* cellularnoise: Function to generate cellular automata-based noise.
* dmi: DMI manipulations which are impossible or degraded from within BYOND.
  Mostly used by the asset cache subsystem to improve load times.
* file: Faster replacements for `file2text` and `text2file`, as well as reading or checking if files exist.
* git: Functions for robustly checking the current git revision.
* http: Asynchronous HTTP(s) client supporting most standard methods.
* json: Function to check JSON validity.
* log: Faster log output.
* noise: 2d Perlin noise.
* sql: Asynchronous MySQL/MariaDB client library.
* time: High-accuracy time measuring.
* toml: TOML parser.
* url: Faster replacements for `url_encode` and `url_decode`.

Additional features are:
* batchnoise: Discrete Batched Perlin-like Noise, fast and multi-threaded - sent over once instead of having to query for every tile.
* hash: Faster replacement for `md5`, support for SHA-1, SHA-256, and SHA-512. Requires OpenSSL on Linux.
* pathfinder: An a* pathfinder used for finding the shortest path in a static node map. Not to be used for a non-static map.
* redis_pubsub: Library for sending and receiving messages through Redis.
* redis_reliablequeue: Library for using a reliable queue pattern through Redis.
* unzip: Function to download a .zip from a URL and unzip it to a directory.
* worleynoise: Function that generates a type of nice looking cellular noise, more expensive than cellularnoise

Regarding rust-analyzer: If you are using a feature set other than the default, you will need to adjust `rust-analyzer.cargo.features`.

## Installing

The rust-g binary (`rust_g.dll` or `librust_g.so`) should be placed in the root
of your repository next to your `.dmb`. There are alternative installation
locations, but this one is best supported.

Compiling will also create the file `target/rust_g.dm` which contains the DM API
of the enabled modules. To use rust-g, copy-paste this file into your project.

`rust_g.dm` can be configured by creating a `rust_g.config.dm`. See the comments
at the top of `rust_g.dm` for details.

## Troubleshooting

You must build a 32-bit version of the library for it to be compatible with
BYOND. Attempting to build a 64-bit version will fail with an explanatory error.

### Linux

On Linux systems `ldd` can be used to check that the relevant runtime libraries
are installed, without which BYOND will fail to load rust-g. The following is
sample output, but the most important thing is that nothing is listed as
"missing".

```sh
$ ldd librust_g.so  # Linux
    linux-gate.so.1 (0xf7f8b000)
    libgcc_s.so.1 => /lib/i386-linux-gnu/libgcc_s.so.1 (0xf7957000)
    libpthread.so.0 => /lib/i386-linux-gnu/libpthread.so.0 (0xf7935000)
    libm.so.6 => /lib/i386-linux-gnu/libm.so.6 (0xf7831000)
    libdl.so.2 => /lib/i386-linux-gnu/libdl.so.2 (0xf782b000)
    libc.so.6 => /lib/i386-linux-gnu/libc.so.6 (0xf7643000)
    /lib/ld-linux.so.2 (0xf7f8d000)
```

If BYOND cannot find the shared library, ensure that the directory containing
it is included in the `LD_LIBRARY_PATH` environment variable, or tweak the search
logic in `rust_g.dm`:

```sh
$ export LD_LIBRARY_PATH=/path/to/tgstation
```

To examine what locations BYOND is searching for the shared library, use
`strace`:

```sh
$ strace DreamDaemon tgstation.dmb 45000 -trusted -logself 2>&1 | grep 'rust_g'
# Early in output, the file will be listed when BYOND examines every file it can see:
open("rust_g", O_RDONLY|O_NONBLOCK|O_LARGEFILE|O_DIRECTORY|O_CLOEXEC) = -1 ENOTDIR (Not a directory)
# BYOND will then search some common directories...
stat64("/home/game/.byond/bin/rust_g", 0xffef1110) = -1 ENOENT (No such file or directory)
stat64("/home/game/.byond/bin/rust_g", 0xffef1190) = -1 ENOENT (No such file or directory)
# Then anywhere in LD_LIBRARY_PATH...
open("/home/game/work/ss13/byond/bin/rust_g", O_RDONLY|O_CLOEXEC) = -1 ENOENT (No such file or directory)
# Then in several interesting places where ld-linux looks...
open("tls/i686/sse2/cmov/rust_g", O_RDONLY|O_CLOEXEC) = -1 ENOENT (No such file or directory)
    ... snip ...
open("cmov/rust_g", O_RDONLY|O_CLOEXEC) = -1 ENOENT (No such file or directory)
# Until finding the library fails or succeeds (a value other than -1 indicates success):
open("rust_g", O_RDONLY|O_CLOEXEC)      = 4
# After that it goes back to the scanning from startup.
open("rust_g", O_RDONLY|O_NONBLOCK|O_LARGEFILE|O_DIRECTORY|O_CLOEXEC) = -1 ENOTDIR (Not a directory)
```

If you're still having problems, ask in the [Coderbus Discord]'s
`#tooling-questions` channel.

[/tg/station]: https://github.com/tgstation/tgstation
[Rust]: https://rust-lang.org
[Cargo]: https://doc.rust-lang.org/cargo/
[rustup]: https://rustup.rs/
[msvc]: https://visualstudio.microsoft.com/thank-you-downloading-visual-studio/?sku=BuildTools&rel=15
[Coderbus Discord]: https://discord.gg/Vh8TJp9

## License

This project is licensed under the [MIT license](https://en.wikipedia.org/wiki/MIT_License).

See [LICENSE](./LICENSE) for more details.
