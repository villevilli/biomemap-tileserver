use image::{DynamicImage, GrayImage, Luma};

pub mod tilecache;

#[derive(Hash, PartialEq, Eq, Clone, Copy, Debug)]
pub struct TilePos {
    pub zoom: i32,
    pub x: i32,
    pub y: i32,
}

impl TilePos {
    pub fn new(zoom: i32, x: i32, y: i32) -> Self {
        Self { zoom, x, y }
    }
}

/// The image should be 256x256
pub trait TileProvider {
    fn get_tile(&self, pos: TilePos) -> Option<DynamicImage>;
}

#[derive(Default)]
pub struct Blacktile;

impl TileProvider for Blacktile {
    fn get_tile(&self, _: TilePos) -> Option<DynamicImage> {
        Some(GrayImage::from_pixel(256, 256, Luma::from([0])).into())
    }
}

impl Blacktile {
    pub const fn new() -> Self {
        Self
    }
}
