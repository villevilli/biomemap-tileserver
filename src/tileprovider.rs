use image::{DynamicImage, GrayImage, Luma};

/// The image should be 256x256
pub trait TileProvider {
    fn get_tile(&self, zoom: i32, x: i32, y: i32) -> DynamicImage;
}

pub struct Blacktile;

impl TileProvider for Blacktile {
    fn get_tile(&self, _: i32, _: i32, _: i32) -> DynamicImage {
        GrayImage::from_pixel(256, 256, Luma::from([0])).into()
    }
}

#[allow(dead_code)]
impl Blacktile {
    pub const fn new() -> Self {
        Self
    }
}
