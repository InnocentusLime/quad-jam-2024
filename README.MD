## What is this

This is a Zelda-like, Souls-like game about some bunny.
It is in active development.

## Controls

* Space -- start / reset on GameOver
* WASD/arrow keys -- Move around
* Mouse -- aim
* Left Mouse Button -- attack
* F11 -- toggle fullscreen
* Escape -- pause

## Building For Deploy

1. Make sure you have docker installed (minimal version: `28.2.2`)
2. Clone this repository
3. Run `docker build -t quad-jam .` in the repository root
4. Run `docker run --rm -p 8080:80 --name quad-jam quad-jam`
    * NOTE: the command will hang. This is intentional. Pressing Ctrl+C will
      stop the game
5. You can now play the game by going to `localhost:8080` in your browser of choice
6. To stop the game just go back to the terminal and press Ctrl+C

## Building For Development

1. Make sure you have the reqiured dependencies.
    * (For Linux) make sure you have the development packages of the following libraries installed: `asound2, udev, wayland, xkbcommon`.
    * (For Windows) make sure you have Windows SDK installed.
2. Make sure you have Rust version 1.88.0 or higher.
3. Make sure you have Cargo version 1.88.0 or higher.
4. Run the following command in project root.
```
cargo build
```
5. To run the dev build, execute the following ocmmand in project root.
```
cargo run
```

## Working With The Project

1. Make sure you can build both targets according to the `Building` section
2. Make sure you have [Tiled](mapeditor.org) version **1.10**

## Credits

* Font "quaver". Can be downloaded from [the itch.io page](https://caffinate.itch.io/quaver)
* Tileset by piixl. Can be found on [the itch.io page](https://piiixl.itch.io/1-bit-patterns-and-tiles)