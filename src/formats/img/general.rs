use std::io::BufReader;

use crate::{formats::Format, Path};

use image::DynamicImage;
pub use image::ImageFormat;
use reerror::Result;

/// Wrapper around using the `image` crate to parse images
pub struct Img(pub ImageFormat);

impl Format for Img {
    type Output = DynamicImage;
    fn parse(&self, path: &Path) -> Result<Self::Output> {
        let reader = BufReader::new(path.reader()?);

        Ok(image::load(reader, self.0)?)
    }
}
