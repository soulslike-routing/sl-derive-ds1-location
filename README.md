# SL-Derive Template
## Abstract
Template repo, for creating user-supplied wasm-functions for the soulslike-routing project.
Also includes a small webapp for testing purposes, where you can check, if your derive function
works / does what you think it does relatively easy.

## About derive functions
So called derive functions are supplied by the user when modeling a game.
Certain kinds of state in the games are difficult or just impossible to just read from the games
memory - They have to be derived from all kinds of different information, like combining parts of the
model, current basic state and previous states.

To not overcomplicate the game-agnostic nature of the SLR-toolchain, I decided for now, that this
kind of functions is to be supplied by the user in the form of small wasm functions when modeling
the game.

## Notes
 - Be careful when changing the package name in Cargo.toml, it is currently hardcoded in a few commands. Check the Taskfile if your plan on using it!
