# RLE and 1bit encoding based image generation
Stores pixel art assets in super small domain-specific implementation of rle and 1bit encoding. These assets are then combined by some rules (or DNA) to generate some final image from any psudo-random value. Useful for unique avatar generation, or NFT avatars.

![screenshot of early version editor](/screenshot.png)

this character can be loaded with the following encoded string: `481220042001216221016041604160016040c060c040600166016122610280628001608660018403a403a002a003a002a002210221`

## Running

Type the following in your terminal in the project directory to build and run a release build: ```cargo run --bin editor --release```

## Components

### Image Editor

see the [editor readme](./EDITOR.md)

TODO:
- add mode that displays RLE size values on canvas pixels
- add fill bucket
- add import and export, open and save


### DNA 2Byte format
    (colorid) (features: (eyes)(alphamask)(colormask))(base_shapes: head, body)
    (3bits  ) (5bits:    (1bit)(2bit     )(2bit     ))(8bits:      (4bit)(4bit))
- first 3 bits from left for color palette id (8 possible options).
- next `1` bit for eye direction (left or right)
<!-- - next `2` bits for alpha mask selection (4 variations: solid, hstripe, dots, or vstripe) -->
- next `2` bits for color mask selection (4 variations: solid, hstripe, dots, or vstripe)
- next `4` bits for head base shape selection (16 options)
- next `4` bits for body base shape selection (16 options)
- might possibly do 3 bytes for body (4 options) and 5 bytes for head (32 options)

perhaps masks can be indexed calculation functions instead of encoded images. Ex:
```rs
// returns true every other pixel and criss crosses on each line
fn cross_dots(x:u32,y:u32) -> bool {
    return ((x+y) % 2) == 0;
}
// returns true every other line
fn stripes(y) -> bool {
    return y%2 == 0;
}
```


TOTAL DNA BYTES: `2Bytes`

`0b1110110011110000`

### Mask data
Masks will be stored as 1bit-per-pixel encoding with image size 16x16 B/W pixels.
Size: [see encoding breakdown](#1bit-bytes-required)

## Body data

body data will be stored as [rle encoded pixels](#single-byte-rle-encoding)


# Encodings

## Single byte rle encoding

### Header Byte

- width is defined in the first bytes least significant 5 bits (0b000**1_1111**). maximum 32
- width gets 1 added on decode cause 0 size is invalid
- the other 3 bytes will be left offset. Possible offsets: (0 to 7)
maybe we should do a top offset + height byte too?

 Alternative header:
 If our assets dimensions can be assumed on decode (ex 32x32) then the header can be:
`4 bits left offset` + `4 bits top offset`

### Run Length Bytes

for 1bit color: One byte per run. 1 bit decides if the pixel is on or off, and the other 7 bits determine the length of the run. (max 128 run since we will never do a run of 0)

- run length value gets 1 added to it on decode so we don't have 0 length runs
- runs can continue on the next line (wrapping)
- first run starts from top left of image + offset value
- discard rle byte for last row of empty pixels during end step of  encode
- then during decode, infer last rows empty cells until image row reaches width or simply stop drawing pixels

a 4x4 image, with two empty top rows and two filled botom rows, would look like the following:

    0b 0000_0011 --- 0000_0111 ---- 1000_0111
    0x 0x3 --------- none,0x7 ----- fill,0x7
    -> 0off, w4 ---- 8 empty pix -- run of 8 filled pixels

## one-byte rle color
Same as above except:
- run lengths are 5 bits (max run of 32)
- most significant 3 bits declare a color index (0 means empty) leaving 7 color options
empty row: `0b00001111`0xF

a 4x4 image, with two empty top rows and two filled botom rows of color index 7, would look like the following:

    0b 0000_0011 --- 0000_0111 ---- 1110_0111
    0x 0x3 --------- none,0x7 ----- 0x7,0x7
    -> 0off, w4 ---- 8 empty pix -- run of 8 filled pixels



## half-byte rle
- image width fixed at 16px
- run lengths are 3 bits (max run of 16) min of 2
- length equals (len+1)*2 so minimum length unit is 2
- ending empty pixels are discarded. Run ends at last solid pixel.

solid row: `0b1111` 0xF components:
- `0b0111` length (7+1) * 2 = 16
- `0b1000` block = solid

### sbrle bytes required
a rectangle with gaps on each side would take two bytes per line. Any empty or solid lines above or below would only take 1 byte. If the rectangle reaches both sides for 1 or more lines, they will also shrink to 1 byte until it hits a new run.

so an image like this (0 is empty 1 is filled):

```rs

// 1 byte header for image width

[
[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,],
[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,],
// only 1 byte for previous two lines
// 0b_0001_1111 0x31 for 32 length run
[1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,],
// only 1 byte for run of 8 filled pixels
// 0b_1000_0111 0x7 for 8 length run
// msb^ most significant bit is set to indicate pixels are solid
[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,],
[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,],
[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,],
[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,],
[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,],
[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,],
[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,],
[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,],
[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,],
[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,],
[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,],
[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,],
[0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,0,]
// final 13 rows = 13x16=208 pixels
// 0b_0111_1111 0x7F for 128 length run leaving 80 pixels remaining
// 0b_0100_1111 0x4F for 80 length run
// these two bytes are discarded, and decoder infers empty
]

```
^ has a total byte count of 5. However, this is an extreme example since most images are more detailed than this one. After empty row discard this image is actually 3 bytes.



## Sq 1bit encoding
1bit per pixel encoding that is always for a multiple of 8 in both directions. 

- no dimensions are stored in the data structure, the width and height are always assumed to be: `((byte_count*8)/2)/8` or `sqrt(byte_count)`

### 1bit bytes required
*for a 16x16 image:*
each bit from left to right is equal to a boolean (1bit) pixel. Either on or off.
For a 16x16 image that takes `2*8 bits per row * 16 rows` = `512bits` or `64Bytes` per mask
with 3 masks that is `192 Bytes` of storage. This will matter in the case of blockchain, since storage is expensive.