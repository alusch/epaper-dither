use std::{fs::File, io::Write};

use anyhow::{anyhow, Context, Result};
use exoquant::{
    ditherer::{Ditherer, FloydSteinberg},
    Color, ColorSpace, Remapper, SimpleColorSpace,
};
use image::{io::Reader, ImageBuffer, Rgb};
use lazy_static::lazy_static;

use crate::image_info::ImageInfo;

const fn color(r: u8, g: u8, b: u8) -> Color {
    Color { r, g, b, a: 255 }
}

const PALETTE: &[Color] = &[
    color(0, 0, 0),       // Black
    color(255, 255, 255), // White
    color(67, 138, 28),   // Green
    color(100, 64, 255),  // Blue
    color(191, 0, 0),     // Red
    color(255, 243, 56),  // Yellow
    color(232, 126, 0),   // Orange
];

const WIDTH: u32 = 600;
const HEIGHT: u32 = 448;

// Default is 2.2, but bumping it up slightly to get a bit more contrast.
const DITHER_GAMMA: f64 = 2.3;

lazy_static! {
    static ref COLOR_SPACE: SimpleColorSpace = SimpleColorSpace {
        dither_gamma: DITHER_GAMMA,
        ..Default::default()
    };
    static ref DITHERER: FloydSteinberg = FloydSteinberg::new();
    pub static ref REMAPPER: Remapper<'static, SimpleColorSpace, FloydSteinberg> =
        Remapper::new(PALETTE, &COLOR_SPACE, &DITHERER);
}

/// Given an image mapping, dithers the image and saves it to the output location.
/// Optionally saves a PNG preview alongside it.
pub fn dither_image<C: ColorSpace, D: Ditherer>(
    info: &ImageInfo,
    remapper: &Remapper<C, D>,
    png: bool,
) -> Result<()> {
    let img = Reader::open(&info.input)
        .with_context(|| format!("Failed to open image {:?}", info.input))?
        .decode()
        .with_context(|| format!("Failed to decode image {:?}", info.input))?
        .to_rgb8();

    let width = img.width();
    let height = img.height();
    if img.width() != WIDTH || height != HEIGHT {
        return Err(anyhow!(
            "Skipping {:?} with dimensions {}x{}",
            info.input,
            width,
            height
        ));
    }

    let pixels: Vec<_> = img.pixels().map(|p| color(p[0], p[1], p[2])).collect();
    let dithered = remapper.remap(&pixels, width as usize);
    let bytes: Vec<_> = dithered.chunks(2).map(|x| x[0] << 4 | x[1]).collect();

    let mut file = File::create(&info.output)
        .with_context(|| format!("Failed to create output file {:?}", info.output,))?;
    file.write_all(&bytes)
        .with_context(|| format!("Failed to write output file {:?}", info.output,))?;

    // If requested, map the dithered index values back to the palette colors and save a PNG
    // for preview purposes without having to load the output files on the frame.
    if png {
        let rgb: Vec<_> = dithered
            .iter()
            .flat_map(|i| {
                let color = PALETTE[*i as usize];
                [color.r, color.g, color.b]
            })
            .collect();
        let rgb_img = ImageBuffer::<Rgb<u8>, Vec<_>>::from_vec(width, height, rgb).unwrap();
        let png_file = info.output.with_extension("png");
        rgb_img
            .save(&png_file)
            .with_context(|| format!("Failed to write PNG file {:?}", png_file))?;
    }

    Ok(())
}
