# epaper-dither

A tool to convert images for display on a [WaveShare 5.65" 7-color E-Paper display](https://www.waveshare.com/product/displays/e-paper/epaper-1/5.65inch-e-paper-module-f.htm), written in Rust.

## Usage

```
epaper-dither <IMAGE>... -o <output> [-p] [-r]
```

At its core, this tool reads input images of a variety of formats, dithers them with Floyd-Steinberg to the E-Paper's palette, and saves output files containing the raw image bytes to send to the display.

In particular, it's designed for my [E-Paper Picture Frame](https://github.com/alusch/EPaperPictureFrame) where I want to put a bunch of images on an SD card to display on the frame and might want to go back and add some more later, but at the end so that everything is seen once before looping around. This is handled with a 4-digit index that is prepended to the output files. Images that already exist in the output directory (ignoring the index prefix) are overwritten.

Input images should be sized to the display's resolution of 600 x 448; anything not that size will be skipped. Globs (e.g. `*.jpg`) are supported on Windows where the shell doesn't expand them automatically.

### Flags

* `-r, --random` : the input images are shuffled prior to assigning indices to randomize the order. 
* `-p, --png` : writes a PNG preview alongside the output binary file to see how the dithering turned out without viewing it on the display.

## Examples

### In order, no existing images

```
epaper-dither -o foo apple.jpg banana.jpg
```

| Input           | Output existing files | Randomize? | PNG preview? | Output result       |
| --------------- | --------------------- | ---------- | ------------ | ------------------- |
| apple.jpg       | (empty)               | No         | No           | 0001-apple.bin      |
| banana.jpg      |                       |            |              | 0002-banana.bin     |

### In order with some existing images

```
epaper-dither -o foo apple.jpg banana.jpg cantaloupe.jpg date.jpg
```

| Input           | Output existing files | Randomize? | PNG preview? | Output result       |
| --------------- | --------------------- | ---------- | ------------ | ------------------- |
| apple.jpg       | 0001-banana.bin       | No         | No           | 0001-banana.bin     |
| banana.jpg      | 0002-date.bin         |            |              | 0002-date.bin       |
| cantaloupe.jpg  |                       |            |              | 0003-apple.bin      |
| date.jpg        |                       |            |              | 0004-cantaloupe.bin |

### Randomized with PNG previews

```
epaper-dither -o foo -p -r apple.jpg banana.jpg cantaloupe.jpg date.jpg
```

| Input           | Output existing files | Randomize? | PNG preview? | Output result       |
| --------------- | --------------------- | ---------- | ------------ | ------------------- |
| apple.jpg       | (empty)               | Yes        | Yes          | 0001-date.bin       |
| banana.jpg      |                       |            |              | 0001-date.png       |
| cantaloupe.jpg  |                       |            |              | 0002-apple.bin      |
| date.jpg        |                       |            |              | 0002-apple.png      |
|                 |                       |            |              | 0003-cantaloupe.bin |
|                 |                       |            |              | 0003-cantaloupe.png |
|                 |                       |            |              | 0004-banana.bin     |
|                 |                       |            |              | 0004-banana.png     |

## License

Distributed under the [MIT license].

[MIT license]: /LICENSE
