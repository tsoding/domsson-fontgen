# Fontgen

A simple tool that compresses bitmap fonts created by
[domsson](https://opengameart.org/users/domsson) with a simple custom
compression algorithm. Compresses those specific fonts better than PNG.

Originally created for my experimental [rust-browser-game](https://github.com/tsoding/rust-browser-game).

## Expected parameters of the font

- A png image file 128x64 pixels
- Monochrome image with black for the background and white for the foreground

## Compression comparison with PNG

<!-- TODO: Document Compression comparison with PNG -->
TBD

## Compression/Decompression Algorithms

<!-- TODO: Document Compression/Decompression Algorithms -->
TBD

## Quick Start

```console
$ make
$ ./fontgen ./charmap-oldschool_white.png
```
