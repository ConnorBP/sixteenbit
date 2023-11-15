use bytemuck::Pod;

use crate::types::{ColorIndex, IndexedImage};


/// Structure representing an image encoded with my Domain Specific 1Byte-per-run Color RLE encoding
#[derive(Clone)]
struct OneByteRle {
    bytes: Vec<u8>,
}

impl<const N: usize, const W: usize> From<IndexedImage<N,W>> for OneByteRle {
    fn from(value: IndexedImage<N,W>) -> Self {
        todo!()
    }
}

pub fn indexed_to_rle<const PIXELS: usize, const WIDTH: usize>(image: IndexedImage<PIXELS, WIDTH>) {

}

// impl From<image>

