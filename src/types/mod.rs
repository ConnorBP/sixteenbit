use std::{ops::{Index, IndexMut}, fmt::Display};


/// A super small 3bit color index
/// Represents a color type we can pick from our selected palette
#[repr(u8)]
#[derive(Debug, Default, Copy, Clone, PartialEq)]
pub enum ColorIndex {
    #[default]
    Empty,
    Dark,
    Bright,
    Skin,
    ShirtAccent1,
    PantsAccent2,
    EyesAccent3,
    Accent4,
}

impl Display for ColorIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
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