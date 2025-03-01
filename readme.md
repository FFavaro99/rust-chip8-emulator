# How to play
Download the latest release and run it.
On start-up, it will ask you to provide a compatible Chip8 program.

The CHIP8 has a total of 15 keys, each representing a hexidecimal value. These keys are organized as such:
```
|1|2|3|C|
---------
|4|5|6|D|
---------
|7|8|9|E|
---------
|A|0|B|F|
```
In this emulator, these keys correspond to:
```
|1|2|3|4|
---------
|Q|W|E|R|
---------
|A|S|D|F|
---------
|Z|X|C|V|
```
Each game uses keys as it pleases, so you'll have to play around with these keys to find out what does what. For space invaders, for example, you move with `Q` and `E` and shoot with `W`

Additionally, you can pause the game with `Spacebar`.

# Multithreading
Multithreading in this program is achieved through mutable shared state. In Rust, this is implemented through Arc<RwLock<T>>. In the future I might consider a refactor in favour of channels.

### These are the threads in this Chip8 implementation:
1. Main -> constitutes the entry point of the emulator, it sets up the shared mutable state of the emulator, loads up a program, and spawns the Emulator and GUI threads;
2. GUI -> responsible with managing the OpenGL objects and rendering the screen;
3. Emulator -> this is the thread that executes all the Chip8 instructions. It is also responsible with spawning the two following threads;
4. Sound Timer -> decrements the sound timer register with a frequency of 60Hz;
5. Delay Timer -> decrements the delay timer register with a frequency of 60Hz;

# Graphics
The graphics are rendered through OpenGL using the [gl-rs](https://github.com/brendanzab/gl-rs.git) bindings. All related code is in the gui.rs source file.

# Features to come
Here is a list of features I am planning to implement:
1. Low-level sound synthesis using [fundsp](https://github.com/SamiPerttu/fundsp.git) and [cpal](https://github.com/RustAudio/cpal.git);
2. Layered OpenGL GUI, to add buttons for loading a new game, pausing, saving game state;
3. A color picker to allow users to choose the color palette they prefer;
4. Debugger capabilities.

# Supported Games

CHIP-8:
- 15 Puzzle by Roger Ivie
- Animal Race by Brian Astle
- Breakout by Carmelo Cortez
- Brick
- Brix by Anderas Gustafsson
- Cave
- Coin Flipping by Carmelo Cortez
- Craps by Carmelo Cortez

S-CHIP-1.1
- Airplane
- Astro Dodge by Revival Studios
- Blinky by Hans Christian Egeberg
- Blitz by David Winter
- Bowling by Gooitzen Van Der Wal
- Breakout by David Winter
- IBM Logo
- - Space Invaders by David Winter