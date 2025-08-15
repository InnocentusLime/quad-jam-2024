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

## Building Manually

Please note the following:
* The desktop build is for development only. The user-ready build
is the WASM one.
* The Linux desktop build should work, but is not tested against all
possible configurations.
* THe MacOS desktop build should work, but is not tested.

### For Desktop (dev build)

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

### For WASM (deploy build)

1. Make sure you have the `wasm32-unknown-unknown` target installed.

2. Create the `dist` directory in project root.
```
mkdir dist
```

3. Create the `levels` inside `dist`
```
mkdir dist/levels
```

4. Build lib-level for desktop.
```
cargo build -p lib-level
```

4. Build all maps. 
```
./target/debug/lib-level --assets ./assets compile-dir -d ./tiled_project -o ./dist/levels
```

5. Build the game under the wasm release profile for wasm.
```
cargo build --target wasm32-unknown-unknown --profile wasm-release --locked
```

6. Put the build into the `dist` folder calling it `game.wasm`
```
cp ./target/wasm32-unknown-unknown/wasm-release/quad-jam-2024.wasm ./dist/game.wasm
```

7. Copy the `assets` folder into `dist` 
```
cp -rf ./assets ./dist/
```

8. Copy the **contents** of `static` folder into `dist` 
```
cp ./static/* ./dist
```

9. You can now deploy the game. The `dist` directory has everything you need.

## Working With The Project

1. Make sure you can build both targets according to the `Building` section
2. Make sure you have [Tiled](mapeditor.org) version **1.10**

## Credits

* Font "oegnek". It can be downloaded from [the itch.io page](https://sonyplaytation.itch.io/oegnec-font) (Unfortunately the link is dead)
* Tileset by piixl. Can be found on [the itch.io page](https://piiixl.itch.io/1-bit-patterns-and-tiles)