# rust-g

rust-g (pronounced rusty-g) is a library which offloads certain expensive or difficult
tasks from BYOND.

This library is currently used in the [tgstation] codebase, and is required for it to run.
A pre-compiled DLL version can be found in the repo root, but you can build your own from
this repo at your preference.

## Compiling

rust-g is build with [Rust] as you might have guessed from the name. Install rust using
[rustup] in order to build rust-g using [cargo] (as a release build, for speed):

    cargo build --release

All dependencies will automatically be downloaded and compiled. rust-g should be compilable on
both Rust stable and nightly.

You will find your `.so` or `.dll` in `targets/release`. More advanced users can
cross-compile rust-g using cargo (but this is beyond the scope of a README).

## Installing

rust-g needs to be installed to your BYOND bin folder (to run in trusted mode), or
the root of your repository (next to your `.dmb`).

Linux users should symlink or rename `librust_g.so` to `rust_g` in order for BYOND
to correctly resolve the library.


[tgstation]: https://github.com/tgstation/tgstation
[Rust]: https://rust-lang.org
[cargo]: https://doc.rust-lang.org/cargo/
[rustup]: https://rustup.rs/

## LICENSE

This project is licensed under the [MIT](https://en.wikipedia.org/wiki/MIT_License) license.

See LICENSE for more details.
