# `ale`
A Rust interface to the [Arcade Learning Environment](https://github.com/mgbellemare/Arcade-Learning-Environment).

Some games such as Breakout, Asteroids, MsPacman and Space Invaders are bundled into the libarary, so that anyone using it can run them. A full list is at https://github.com/trolleyman/ale-rs/blob/master/src/lib.rs#L353-L430.

## Requirements
- CMake (See [cmake-rs](https://github.com/alexcrichton/cmake-rs))

# `ale-sys`
Rust bindings to the [Arcade Learning Environment](https://github.com/mgbellemare/Arcade-Learning-Environment), with a few tweaks. See https://github.com/trolleyman/Arcade-Learning-Environment.

Differences:
- `zlib` is vendored so that compilation is easier
- The C library is statically linked

# `xtask`
`xtask` is a small sub-project used for development. Subcommands can be run by running `cargo xtask <subcommand>` in the root of the repository.

There are two subcommands: `gen-bindings` and `download-roms`.

`gen-bindings` generates the [`ale-sys/src/bindings.rs`](ale-sys/src/bindings.rs) file, and requires clang to be installed.

`download-roms` downloads the bundled Atari ROMs and outputs them in the `roms/` folder, that is then included in the binary via. `include_bytes!`. This is meant to protect me against copyright infringement. It's a similar technique used by [`atari-py`](https://github.com/openai/atari-py).
