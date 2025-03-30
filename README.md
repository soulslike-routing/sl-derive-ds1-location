# Sl-Derive - DS1 current location
## Abstract
Example repo, showing the gist of how user-supplied wasm-functions can be created for the soulslike-routing project.

This example covers a `deriveCurrentLocation` function for Dark Souls: Remastered.

## About derive functions
So called derive functions are supplied by the user when modeling a game.
Certain kinds of state in the games are difficult or just impossible to just read from the games
memory - They have to be derived from all kinds of different information, like combining parts of the
model, current basic state and previous states.

To not overcomplicate the game-agnostic nature of the SLR-toolchain, I decided for now, that this
kind of functions is to be supplied by the user in the form of small wasm functions when modeling
the game.

## Interface
To make interfacing with these kinds of functions, they have a pretty rigid interface you have
to follow when writing them:
At the moment, each function takes the same inputs, consisting of:
- The active spec
- The active model
- The state before this tick
- The state-updates we could already get in this tick.

The functions should then load and parse those objects if needed, compute the new state,
which is probably always a foreign id (uuid) and then also store that in a Memory object, which
is then returned to the javascript caller.
