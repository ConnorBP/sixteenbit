use bevy::log::info;

use crate::types::{ColorIndex, IndexedImage};

// run length must be within 5 bits, so less than 0x20
const RUN_LENGTH_LIMIT: u8 = 0x1 << 5;
const OFFSET_LIMIT: u8 = 0x1 << 3;


/// Structure representing an image encoded with my Domain Specific 1Byte-per-run Color RLE encoding
#[derive(Clone)]
pub struct OneByteRle {
    bytes: Vec<u8>,
}

pub trait Rle {
    fn push();

}

impl OneByteRle {
    pub fn new() -> Self {
        Self {
            bytes: vec![],
        }
    }

    /// return ownership of inner bytes and consume self
    pub fn bytes(self) -> Vec<u8> {
        self.bytes
    }

    /// first byte of the encoding. Sets row width and left offset
    /// does not encode full canvas width. Decoder will make assumptions about width
    /// For my purposes, the assumed width is 32
    pub fn push_header(&mut self, offset: u8, encode_width: u8) {
        // assert that the header is the first byte being entered
        debug_assert_eq!(self.bytes.len(), 0);
        // create header byte
        let header_byte: u8 = (offset as u8) << 5 | (encode_width & 0x1F);
        // push the header to the first byte of our array
        self.bytes.push(header_byte);
    }

    /// Push one run byte to our bytes
    pub fn push_pixel_run(&mut self, pixel_run: &RunByte) {
        // assert that the header is the first byte being entered
        // by now there should be more than one byte
        debug_assert!(self.bytes.len() > 0);
        self.bytes.push(pixel_run.get());
    }

    /// process all of the pixels from a vec of RunBytes
    pub fn append_pixel_runs(&mut self, new_pixel_bytes: &Vec<RunByte>) {
        for pixel in new_pixel_bytes {
            self.push_pixel_run(pixel)
        }
    }
}

/// From implemented for reference so that we don't needlessly clone every pixel before encoding
impl<const N: usize, const W: usize> From<&IndexedImage<N,W>> for OneByteRle {
    fn from(value: &IndexedImage<N,W>) -> Self {
        indexed_to_rle::<N,W>(value)
    }
}

#[derive(Debug,Clone)]
pub struct RunByte {
    color: ColorIndex,// first 3 bits from MSB
    run_length: u8, // last 5 bits closest to LSB
}

impl RunByte {
    pub fn new(color: ColorIndex, run_length: u8) -> Self {
        debug_assert!((color as u8) < (0x1 << 3));
        debug_assert!(run_length < 0x1 << 5);
        Self {
            color,
            run_length,
        }
    }
    pub fn get(&self) -> u8 {
        // return color bits as first 3
        // and RLE as last 5 mased with the lower 5 bits
        (self.color as u8) << 5 | (self.run_length-1 & 0x1F)
    }
}

/// take in an array of indexed colors that make up an image
/// Steps:
/// calculate left offset = x of most left pixel
/// calculate width = (x of most right pixel + 1) - offset
/// (except we don't add the 1 so we can treat 0 as 1 on decode)
/// 
pub fn indexed_to_rle<const PIXELS: usize, const WIDTH: usize>(image: &IndexedImage<PIXELS, WIDTH>) -> OneByteRle {
    // these start as oposite from eachother,
    // then get walked to the right value in the for loop
    let mut min_x = WIDTH as u8-1;
    let mut max_x = 0;
    for (x,_,p) in image.enumerate_pixels() {
        // replace min with current lowest x
        match *p {
            ColorIndex::Empty => {},
            _ => {
                if x < min_x {
                    min_x = x;
                }
                // replace max with current largest x
                if x > max_x {
                    max_x = x;
                }
            }
        }
    }

    // if canvas was empty we set min and max to 0
    if min_x > max_x {
        min_x = 0;
        max_x = 0;
    }



    // now we know our offset value as min_x. Cap it at max 3 bits
    let offset = u8::min(min_x, OFFSET_LIMIT-1);
    // assert that offset value is within 3 bits (max value of 7)
    debug_assert!(offset < 0x1 << 3);
    // and encoded width
    info!("Offset: {offset} max_x: {max_x}");
    let encode_width = max_x - offset;
    // assert that encoded width value is within 5 bits
    // (max value of 31, but we treat zero as 1, so max represented is 32)
    debug_assert!(encode_width < 0x1 << 5);

    info!("actual width: {} encoded_width: {}",encode_width + 1,encode_width);

    // run lengths acumulator
    let mut runs = vec![];

    // the pixel type of the previous cell
    // let mut last_pixel = None;
    for (x,y,p) in image.enumerate_pixels() {

        // skip pixels before the start of offset
        if x < offset { continue; }
        // skip pixels after encode_width + offset
        if x > encode_width + offset { continue; }

        // for first pixel, simply input it into the acu
        if runs.len() == 0 {
            let rb = RunByte::new(
                *p,
                1,
            );
            info!("First run byte: {rb:?}");
            runs.push(rb);
            continue;
        }

        // now the rest of the pixels

        // compare last pixel with current one
        // then either push a new run or increment the last
        match runs.last_mut() {
            Some(last_p) => {
                if last_p.color == *p && last_p.run_length < RUN_LENGTH_LIMIT {
                    // same as last byte
                    // increase the run
                    last_p.run_length+=1;
                } else {
                    // new pixel, push a new run
                    runs.push(RunByte::new(
                        *p,
                        1,
                    ));
                }
            },
            // should never be None since we always push a first pixel
            None => unreachable!(),
        }

        // calculate flat index
        // this won't be needed if I change how my iterator impl to be a flat 1D array
        // let index = 
        //     WIDTH
        //     * y as usize
        //     + x as usize;
        
        
        // if index >= PIXELS-1 {

        // }
    }

    // trunicate trailing null bytes when we reach the last pixel
    // loops until it runs out of colors to check or it hits a non empty color
    loop {
        match runs.last() {
            Some(RunByte {
                color,
                ..
            }) => {
                if *color == ColorIndex::Empty {
                    runs.pop();
                } else {
                    // last run is not an empty,
                    // so stop trunicating 
                    break;
                }
            },
            None => {
                // stop trunicating if we run out of items
                break;
            }
        }
    }

    // convert to final bytes

    let mut out_bytes = OneByteRle::new();

    // push header byte (width and offset)
    out_bytes.push_header(offset, encode_width);

    // push all the run lengths and convert them to bytes
    out_bytes.append_pixel_runs(&runs);

    out_bytes
}

// impl From<image>

