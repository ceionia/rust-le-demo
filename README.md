# Rust on DOS proof of concept (DOS/32A)

this is a demo for using 32-bit Rust on DOS, just very basic. working on something a little bit bigger i might release.

works by producing a ELF binary via Rust, then using a proof of concept utility i wrote to convert the ELF to an LE executable, which DOS/32A can load.

needs the DOS/32A extender to run, compile normally with `cargo build --release` then use `elf2le target/dos/release/rust-le-demo.elf` (or whatever your path is) to produce `a.exe` in the working directory, `new.elf` will be overwritten as an intermediate by `elf2le`, sorry, it's bad code.

you can run the generated executable on DOS with `dos32a a.exe`.

under DOS, the `sc` or `sb` utilities provided by DOS/32A can add a stub to the LE executable, so that you don't need to start it via `dos32a` (`sc` can also compress it).

my elf2le utility can be found on my site: https://ceionia.com/git/lucia/elf2le
or on github: https://github.com/LCeionia/elf2le

more about the project on my site: https://ceionia.com/le-exe

Copyright (c) 2023 Lucia Ceionia

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
