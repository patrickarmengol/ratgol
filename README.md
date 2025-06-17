# ratgol

Conway's Game of Life TUI

## Installation

```bash
cargo install --git https://github.com/patrickarmengol/ratgol
```

## Usage

### Syntax

```bash
ratgol
```

### Controls

| Key           | Function               |
| ------------- | ---------------------- |
| `Esc` or `q`  | quit                   |
| `Space`       | toggle pause/resume    |
| `Up` or `k`   | increase tick interval |
| `Down` or `j` | decrease tick interval |
| `r`           | randomize grid         |
| `c`           | clear grid             |

## TODO

- edit grid with mouse input
- colors
- configuration
- command-line arguments
- load and paste patterns

## Notes

This is just a quick project intended to help me famliarize myself with [`ratatui`](https://ratatui.rs) and practice writing comments. I utilized some elements of the `ratatui` provided event-driven architecture template, but had to make changes for variable tick rates and pausing (probably would have been way easier to go single-threaded for this application).

I also ran into issues with initializing and updating the grid size, since it was tied to the terminal size. The widget pattern means that each widget's area is calculated individually at each new frame, and was not tracked by the App directly, meaning that for init/resize, I needed to calculate the appropriate area ad-hoc using some magic numbers, which would break down if the layout changed.

## License

This project is dual-licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contributing

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
