#![deny(
    missing_copy_implementations,
    missing_debug_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unsafe_code
)]
#![warn(unused_import_braces, unused_qualifications, unused_results)]

use std::path::PathBuf;

use anyhow::Result;
use indicatif::ParallelProgressIterator;
use rayon::prelude::*;
use structopt::StructOpt;

use crate::dither::{dither_image, REMAPPER};
use crate::image_info::get_images;

mod dither;
mod image_info;

/// Tool to convert images for display on a WaveShare 5.65" 7-color E-Paper display.
/// Input images should be 600 x 448 pixels.
#[derive(StructOpt, Debug)]
pub struct Args {
    /// Input image files to be converted
    #[structopt(name = "IMAGE", parse(from_os_str), required = true)]
    sources: Vec<PathBuf>,

    /// Destination folder for E-Paper images
    #[structopt(short, long, parse(from_os_str))]
    output: PathBuf,

    /// Also save PNG previews of the dithered images
    #[structopt(short, long)]
    png: bool,

    /// Randomize order of images that don't already exist in the output directory
    #[structopt(short, long)]
    random: bool,
}

fn main() -> Result<()> {
    let args = Args::from_iter(wild::args());

    let images = get_images(&args.sources, &args.output, args.random)?;
    let errors: Vec<_> = images
        .par_iter()
        .progress_count(images.len() as u64)
        .map(|info| dither_image(info, &REMAPPER, args.png))
        .filter_map(Result::err)
        .collect();

    for error in errors {
        eprintln!("Warning: {}", error);
    }

    Ok(())
}
