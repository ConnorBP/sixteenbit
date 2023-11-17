use std::{ops::{Index, IndexMut}, fmt::Display, os::windows, slice::IterMut};
use bevy::{utils::info, log::info, asset::AsyncReadExt, reflect::TypeData};
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
#[derive(Clone, Zeroable)]
// #[repr(C, packed)]
pub struct IndexedImage<const N: usize, const W: usize> {
    pub vertical_trim: u8,
    resolution: [u8;2],
    pixels: [ColorIndex;N],
}

impl<const N: usize, const W: usize> IndexedImage<N,W> {
    /// shifts all pixels by (x, y) and drops any out of bounds
    pub fn shift(&mut self, x_offset: i32, y_offset: i32) {
        let res = self.resolution;

        let sampler = self.clone();

        for (new_x, new_y,p) in self.enumerate_pixels_mut() {

            let sample_x = (new_x as i32) - x_offset;
            let sample_y = (new_y as i32) - y_offset;

            *p = if
            sample_x >= res[0] as i32
            || sample_x < 0
            || sample_y >= res[1] as i32
            || sample_y < 0
            {
                ColorIndex::Empty
            } else {
                sampler[(sample_x as usize, sample_y as usize)]
            };
        }
    }
}

impl<const N: usize, const W: usize> Default for IndexedImage<N,W> {
    fn default() -> Self {
        Self {
            vertical_trim: 0,
            resolution: [W as u8,(N/W) as u8],
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
            vertical_trim: 0,
            resolution: [W as u8,H as u8],
            pixels: std::array::from_fn::<_,N,_>(|_| ColorIndex::Empty),
        }
    }
    pub fn enumerate_pixels(&self) -> EnumerateIndexedImage<N,W> {
        EnumerateIndexedImage {
            image: self,
            x: 0,
            y: 0,
        }
    }

    pub fn enumerate_pixels_mut(&mut self) -> EnumerateIndexedImageMut<N,W> {
        EnumerateIndexedImageMut {
            image: self,
            x: 0,
            y: 0,
        }
    }
}

impl<const N: usize, const W: usize> Index<(usize, usize)> for IndexedImage<N,W> {
    type Output = ColorIndex;

    fn index(&self, index: (usize, usize)) -> &Self::Output {
        // total pixel count (N) must always be divisible by WIDTH
        debug_assert_eq!(N % W, 0);

        // calculate flat index into the array
        let index = 
            self.resolution[0] as usize // width
            * index.1 as usize // y
            + index.0 as usize; // x

        &self.pixels[index]
    }
}

impl<const N: usize, const W: usize> IndexMut<(usize, usize)> for IndexedImage<N,W> {
    fn index_mut(&mut self, index: (usize, usize)) -> &mut ColorIndex {
        // total pixel count (N) must always be divisible by WIDTH
        debug_assert_eq!(N % W, 0);

        // calculate flat index into the array
        let index = 
            self.resolution[0] as usize // width
            * index.1 as usize // y
            + index.0 as usize; // x

        &mut self.pixels[index]
    }
}

// impl<const N: usize, const W: usize> Iterator for IndexedImage<N,W> {
//     type Item = ColorIndex;

//     #[inline(always)]
//     fn next(&mut self) -> Option<Self::Item> {
//         self.pixels.ne
//     }

//     // #[inline(always)]
//     // fn size_hint(&self) -> (usize, Option<usize>) {
//     //     let len = self.len();
//     //     (len, Some(len))
//     // }
// }

/// for enumerating pixel contents with ease.
/// Based on the enumerate pixels system of the image crate.
pub struct EnumerateIndexedImage<'a, const N: usize, const W: usize> {
    image: &'a IndexedImage<N,W>,
    x: u8,
    y: u8,
}

impl<'a, const N: usize, const W: usize> Iterator for EnumerateIndexedImage<'a,N,W> {
    type Item = (u8,u8, &'a ColorIndex);

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.x >= self.image.resolution[0] {
            self.x = 0;
            self.y += 1;
        }
        let (x, y) = (self.x, self.y);
        self.x += 1;
        
        // calculate flat index into the array
        let index = 
            self.image.resolution[0] as usize // width
            * y as usize
            + x as usize;

        // stop when we run out of pixels
        if index >= N {
            return None;
        }
        
        // info!("enumerating pixel {x} {y}");
        Some((x,y,&self.image[(x as usize, y as usize)]))
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.image.resolution[0] as usize * self.image.resolution[1] as usize;
        (len, Some(len))
    }
}

pub struct EnumerateIndexedImageMut<'a, const N: usize, const W: usize>
{
    image: &'a mut IndexedImage<N,W>,
    x: u8,
    y: u8,
}

impl<'a, const N: usize, const W: usize> Iterator for EnumerateIndexedImageMut<'a,N,W> {
    type Item = (u8,u8, &'a mut ColorIndex) where Self::Item: 'a;

    #[inline(always)]
    fn next(& mut self) -> Option<Self::Item>
    {

        if self.x >= self.image.resolution[0] {
            self.x = 0;
            self.y += 1;
        }
        let (x, y) = (self.x, self.y);
        self.x += 1;
        
        // calculate flat index into the array
        let index = 
            self.image.resolution[0] as usize // width
            * y as usize
            + x as usize;

        // stop when we run out of pixels
        if index >= N {
            return None;
        }

        let ptr = self.image.pixels.as_mut_ptr();
        
        // info!("enumerating pixel {x} {y}");
        // std::mem::take(&mut self.image[(x as usize, y as usize)])

        // calculate flat index into the array
        let index = 
            self.image.resolution[0] as usize // width
            * y as usize // y
            + x as usize; // x

        // unsafe because of this:
        // https://stackoverflow.com/questions/63437935/in-rust-how-do-i-create-a-mutable-iterator
        unsafe {
            Some((
                x,
                y,
                &mut *ptr.add(index)
            ))
        }
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.image.resolution[0] as usize * self.image.resolution[1] as usize;
        (len, Some(len))
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