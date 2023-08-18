# Game of life using Fully homomorphic encryption

This is a very simple implementation of Conway's [Game of Life](https://en.wikipedia.org/wiki/Conway%27s_Game_of_Life) using periodic boundary conditions, with a twist: the board is encrypted and all calculations are performed in encrypted space. In principle, calculations can thus be done by a thread or server which has no access to the state of the board.

## Build and run

```bash
cargo run --release --bin game-of-life
cargo run --release --bin game-of-life --features fpga
```
