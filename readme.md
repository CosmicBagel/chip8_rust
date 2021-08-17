# Chip8_rust

A chip 8 emulator written in rust

Currently it is functional, but is missing sound, and has some timing bugs.

## Thanks to
- https://github.com/kripod/chip8-roms for providing a nice collection of roms to test with
- http://devernay.free.fr/hacks/chip8/C8TECH10.HTM#2.5 solid technical reference I used
- The maintainers of https://en.wikipedia.org/wiki/CHIP-8 which I also used as technical reference
- https://ajor.co.uk/chip8/chip8.html for a nice emulator to compare behaviour against
- https://github.com/corax89/chip8-test-rom the first test rom I used
- https://github.com/Skosulor/c8int/tree/master/test the second more thorough test rom I used

## Input
```
    keyboard mapping:
    Keypad      Keyboard
    1 2 3 C     1 2 3 4
    4 5 6 D ->  Q W E R
    7 8 9 E     A S D F
    A 0 B F     Z X C V
 ```
 Eg. Pong uses 1 and 4 to move the paddels, on the keyboard this is 1 and Q

## Building

Just clone and `cargo run`

*Should* work on windows, linux, and macos

See https://github.com/kripod/chip8-roms for roms to try

¯\\\_(ツ)_/¯

