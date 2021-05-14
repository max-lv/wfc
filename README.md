
# About

Yet another Rust implementation of Wave Function Collapse (https://github.com/mxgmn/WaveFunctionCollapse).

This is only a tile-mode, no texture synthesis.

Made solely as a learning project don't use it :P

Features:

- 2d/3d generation
- Big-tiles of arbitrary shape (can have holes)
- Backtracking (only for a single step, but so-far on my tilesets that makes this 100% fail free, tested for 10000 generations)
- Very slow (due to memory allocating like there is no tomorrow)

# Usage

```
$ git clone ...
# this is a known working commit
$ git checkout a37b2062
$ cargo run --release
```

This will open SDL window with interactable generator on pipes tileset.

Key-bindings:

- **F** - single WFC step
- **Q** - auto-collapse untill success / error
- **N** - restart and use new seed
- **R** - restart current seed

