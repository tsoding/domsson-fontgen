# Fontgen

A simple tool that compresses bitmap fonts created by
[domsson](https://opengameart.org/users/domsson) with a simple custom
compression algorithm. Compresses those specific fonts better than PNG.

Originally created for my experimental [rust-browser-game](https://github.com/tsoding/rust-browser-game).

## Expected parameters of the font

- A png image file 128x64 pixels
- Monochrome image with black for the background and white for the foreground

## Compression comparison with PNG

| File Name                                                          | Original PNG size (bytes) | Size of `./fontgen -f bin` output (bytes) |
| ---                                                                | ---                       | ---                                       |
| [charmap-cellphone_white_0.png](./charmap-cellphone_white_0.png)   | 1103                      | 623                                       |
| [charmap-futuristic_black_0.png](./charmap-futuristic_black_0.png) | 1070                      | 621                                       |
| [charmap-oldschool_white.png](./charmap-oldschool_white.png)       | 1026                      | 622                                       |


## Compression/Decompression Algorithms

<!-- TODO: Document Compression/Decompression Algorithms -->
TBD

## Quick Start

```console
$ make
$ ./fontgen ./charmap-oldschool_white.png
```
