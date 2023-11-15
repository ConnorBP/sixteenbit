use std::{ops::{Index, IndexMut}, fmt::Display, os::windows};
use bytemuck::{Zeroable, Pod, Contiguous};
use static_assertions::{const_assert_eq, const_assert};


/// A super small 3bit color index
/// Represents a color type we can pick from our selected palette
#[repr(u8)]
#[derive(Debug, Default, Copy, Clone, PartialEq)]
// #[transparent(u8)]
pub enum ColorIndex {
    #[default]
    Empty = 0,
    Dark = 1,
    Bright = 2,
    Skin = 3, // Might rename this to MidTone to be more generic
    ShirtAccent1 = 4,
    PantsAccent2 = 5,
    EyesAccent3 = 6,
    Accent4 = 7,
}

// we are treating the enum as a raw u8
// so this just double checks that the compiler is in fact treating it as such
static_assertions::assert_eq_size!(ColorIndex,u8);

unsafe impl Zeroable for ColorIndex {
    fn zeroed() -> Self {
        unsafe { core::mem::zeroed() }
    }
}
unsafe impl Pod for ColorIndex {}

unsafe impl Contiguous for ColorIndex {
    type Int = u8;
    const MAX_VALUE: u8 = ColorIndex::Accent4 as u8;
    const MIN_VALUE: u8 = ColorIndex::Empty as u8;
}

impl Display for ColorIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// Representation of the Non-Encoded pixel bytes that are in the intermediary indexed format already.
#[derive(Copy, Clone, Zeroable)]
// #[repr(C, packed)]
pub struct IndexedImage<const N: usize, const W: usize> {
    resolution: [u8;2],
    pixels: [ColorIndex;N],
}

impl<const N: usize, const W: usize> Default for IndexedImage<N,W> {
    fn default() -> Self {
        Self {
            resolution: Default::default(),
            pixels: std::array::from_fn::<_,N,_>(|_| ColorIndex::Empty)
        }
    }
}



impl<const N: usize,const W: usize> IndexedImage<N,W> {
    pub fn new<const H: usize>() -> Self {
        // panic if the inputted pixel count is not the same as array size
        // sadly static assertions are not working here
        assert_eq!(W as usize * H as usize, N);
        IndexedImage {
            resolution: [W as u8,H as u8],
            pixels: [ColorIndex::Empty;N],
        }
    }
}

impl<const N: usize, const W: usize> Index<usize> for IndexedImage<N,W> {
    type Output = [ColorIndex];

    fn index(&self, index: usize) -> &Self::Output {
        // total pixel count (N) must always be divisible by WIDTH
        debug_assert_eq!(N % W, 0);
        &self.pixels
        .chunks(W)
        .nth(index)
        // .map(|r|PixelRow(r.to_vec()) )
        .expect("getting pixel row")
        // .to_vec()
    }
}

impl<const N: usize, const W: usize> IndexMut<usize> for IndexedImage<N,W> {
    fn index_mut(&mut self, index: usize) -> &mut [ColorIndex] {
        // total pixel count (N) must always be divisible by WIDTH
        debug_assert_eq!(N % W, 0);
        self.pixels
        .chunks_mut(W)
        .nth(index)
        .expect("getting pixel row")
    }
}

// collection of pallets (max 8)
pub struct PaletteCollection<T> {
    palettes: [Palette<T>;8]
}

impl Default for PaletteCollection<u8> {
    fn default() -> Self {
        Self { palettes: Default::default() }
    }
}

impl<T> Index<u8> for PaletteCollection<T> {
    type Output = Palette<T>;

    fn index(&self, index: u8) -> &Self::Output {
        self.palettes.index(index as usize)
    }
}

impl<T> IndexMut<u8> for PaletteCollection<T> {
    fn index_mut(&mut self, index: u8) -> &mut Self::Output {
        self.palettes.index_mut(index as usize)
    }
}

pub struct Palette<T> {
    contents: [image::Rgb<T>;7],
}

impl<T> Index<ColorIndex> for Palette<T> {
    type Output = image::Rgb<T>;

    /// fails if you try to index EmptyBrush
    fn index(&self, index: ColorIndex) -> &Self::Output {
        &self.contents[index as usize -1]
    }
}

impl Default for Palette<u8> {
    fn default() -> Self {
        Self { contents: [
            image::Rgb([0,0,0]),
            image::Rgb([255,255,255]),
            image::Rgb([204,164,153]),
            image::Rgb([255,165,96]),
            image::Rgb([101,107,255]),
            image::Rgb([173,101,255]),
            image::Rgb([62,24,24]),
        ] }
    }
}