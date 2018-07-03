# rust-g

rust-g (pronounced rusty-g) is a library which offloads certain expensive or difficult
tasks from BYOND.

This library is currently used in the [tgstation] codebase, and is required for it to run.
A pre-compiled DLL version can be found in the repo root, but you can build your own from
this repo at your preference.

## Dependencies

The [Rust] compiler:

1. Use [the Rust installer](https://rustup.rs/), or another Rust installation method,
   or run this command:

    ```sh
    curl https://sh.rustup.rs -sSf | sh -s -- --default-toolchain stable
    ```

1. Set the default compiler to **32-bit**:

    ```sh
    # in the `rust-g` directory...
    cd rust-g
    # Linux
    rustup override add stable-i686-unknown-linux-gnu
    # Windows
    rustup override add stable-i686-pc-windows-msvc
    ```

OpenSSL (**Optional**, not required by the default configuration):

* Ubuntu and Debian users run:

    ```sh
    sudo apt-get install libssl-dev:i386 libssl1.0.0:i386 pkg-config gcc-multilib
    ```

* Other distributions install the appropriate **32-bit development** and **32-bit runtime** packages.

## Compiling

The [cargo] tool handles compilation, as well as automatically downloading and
compiling all Rust dependencies. The default configuration is suitable for
use with the [tgstation] codebase. To compile in release mode (recommended for
speed):

```sh
cargo build --release
```

On Linux, the output will be `target/release/librust_g.so`, but **must be renamed**
to `rust_g` to install correctly.

On Windows, the output will be `target/release/rust_g.dll`.

For more advanced configuration, a list of modules may be passed:

```sh
cargo build --release --features dmi,file,log,url
```

* **dmi** (default): DMI manipulations which are impossible from within BYOND.
  Used by the asset cache subsystem to improve load times.
* file: Faster replacements for `file2text` and `text2file`.
* hash: Faster replacement for `md5`, support for SHA-1, SHA-256, and SHA-512. Requires OpenSSL on Linux.
* **log** (default): Faster log output.
* url: Faster replacements for `url_encode` and `url_decode`.

## Installing

The rust-g binary needs to be copied to either your BYOND bin folder, or to the
root of the repository (next to your `.dmb`).

On Linux, be sure the file is named `rust_g`.

Compiling will also create the file `target/rust_g.dm` which contains the DM API
of the enabled modules. To use rust-g, copy-paste this file into your project.

It is also possible to automatically override the built-in versions of the
functions being replaced, though this ability is not well-tested. To enable
this, create and include `rust_g.config.dm` in the same directory you placed
`rust_g.dm`, with the contents:

```dm
#define RUSTG_OVERRIDE_BUILTINS
```

## Troubleshooting

The most common mistake is building a 64-bit version of the library, which BYOND
will be unable to load. If the output of the `file` command indicates a 64-bit
build, make sure you are using the 32-bit Rust compiler. Correct output for a
32-bit build will look similar to:

```sh
$ file rust_g  # Linux
ELF 32-bit LSB shared object, Intel 80386, version 1 (SYSV), dynamically linked, BuildID[sha1]=..., not stripped

$ file rust_g.dll  # Windows
PE32 executable (DLL) (GUI) Intel 80386 (stripped to external PDB), for MS Windows
```

On Linux systems where the `hash` module is in use, `ldd` can be used to check
that the OpenSSL runtime libraries are installed, without which BYOND will fail
to load rust-g. Use the `ldd` command to check that the dependencies are being
found, and no libraries are missing:

```sh
$ ldd rust_g  # Linux
        linux-gate.so.1 =>  (0xf7775000)
        libssl.so.1.0.0 => /lib/i386-linux-gnu/libssl.so.1.0.0 (0xf7677000)
        libcrypto.so.1.0.0 => /lib/i386-linux-gnu/libcrypto.so.1.0.0 (0xf748a000)
        libdl.so.2 => /lib/i386-linux-gnu/libdl.so.2 (0xf7485000)
        librt.so.1 => /lib/i386-linux-gnu/librt.so.1 (0xf747c000)
        libpthread.so.0 => /lib/i386-linux-gnu/libpthread.so.0 (0xf745e000)
        libgcc_s.so.1 => /lib/i386-linux-gnu/libgcc_s.so.1 (0xf7441000)
        libc.so.6 => /lib/i386-linux-gnu/libc.so.6 (0xf728b000)
        /lib/ld-linux.so.2 (0x5657d000)
        libm.so.6 => /lib/i386-linux-gnu/libm.so.6 (0xf7236000)
```

If you're still having problems, ask in [tgstation]'s IRC, `#coderbus` on Rizon.

[tgstation]: https://github.com/tgstation/tgstation
[Rust]: https://rust-lang.org
[cargo]: https://doc.rust-lang.org/cargo/
[rustup]: https://rustup.rs/

## License

This project is licensed under the [MIT license](https://en.wikipedia.org/wiki/MIT_License).

See [LICENSE](./LICENSE) for more details.
