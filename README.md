# Rust LE Demo

this is a demo for using 32-bit Rust on DOS, just very basic. working on something a little bit bigger i might release.

needs the DOS/32A extender to run, compile normally with `cargo build --release` then use `elf2le target/dos/release/rust-le-demo.elf` (or whatever your path is) to produce `a.exe` in the working directory, `new.elf` will be overwritten as an intermediate by `elf2le`, sorry, it's bad code.

you can run the generated executable on DOS with `dos32a a.exe`.

my elf2le utility can be found on my site: https://ceionia.com/git/lucia/elf2le
or on github: https://github.com/LCeionia/elf2le

more about the project on my site: https://ceionia.com/le-exe
