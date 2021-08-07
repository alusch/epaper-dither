use std::{
    fmt::Display,
    fs::{metadata, read_dir},
    path::{Path, PathBuf},
};

use anyhow::{anyhow, Context, Result};
use atoi::atoi;
use itertools::Itertools;
use rand::prelude::*;
use rayon::prelude::*;
use regex::{escape, Regex};

/// Mapping between input and output (dithered) image files.
#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ImageInfo<'a> {
    /// The input image file path (e.g. foo.jpg)
    pub input: &'a Path,

    // The output dithered binary file (e.g. 0001-foo.bin)
    pub output: PathBuf,
}

/// Helper trait to simplify working with iterators of results.
trait ResultIterator<T, E>: Iterator<Item = Result<T, E>> + Sized
where
    E: Display,
{
    /// Returns only the `Ok` results and logs the failures to stderr.
    fn log_errors_and_collect(self) -> Vec<T> {
        let (oks, errors): (Vec<_>, Vec<_>) = self.partition_result();
        for error in errors {
            eprintln!("Warning: {}", error);
        }
        oks
    }
}

impl<I, T, E> ResultIterator<T, E> for I
where
    I: Iterator<Item = Result<T, E>>,
    E: Display,
{
}

/// Given an input glob and output directory, return the mappings of input file to output dithered image.
/// Optionally randomizes the input files.
pub fn get_images<'a>(
    sources: &'a [PathBuf],
    destination: &Path,
    random: bool,
) -> Result<Vec<ImageInfo<'a>>> {
    // Verify the source files exist and randomize if desired.
    let mut source_files = sources
        .iter()
        .map(|r| {
            metadata(r)
                .with_context(|| format!("Source file {:?} does not exist", r))
                .map(|_| r)
        })
        .log_errors_and_collect();

    if random {
        let mut rng = thread_rng();
        source_files.shuffle(&mut rng);
    }

    let dest_iter = read_dir(destination)
        .with_context(|| format!("Failed to open destination directory {:?}", destination))?;

    // Find all files in the destination directory, ignoring those that can't be represented as `String`s
    // in order to keep things simple and make it easy to work with them.
    let dest_files = dest_iter
        .map(|r| {
            r.context("Failed to get file info")?
                .file_name()
                .into_string()
                .map_err(|e| anyhow!("Unsupported filename {:?}", e))
        })
        .log_errors_and_collect();

    // Files in the destination directory are expected to be in the form NNNN-file.bin,
    // where N is a decimal digit. Parse those and find the largest index of any existing file,
    // then create an infinite iterator starting at the next number that we can use to assign
    // indices to the new files. This gives us an easy way to add new files without affecting
    // the existing ordering, as well as making randomization easy.
    let last_index = dest_files
        .par_iter()
        .filter_map(|s| atoi::<u32>(s.as_bytes()))
        .max()
        .unwrap_or(0);
    let mut indices = last_index + 1..;

    // Map each source file to an output file, overwriting an existing one with the same name if present,
    // otherwise using the next free index at the end.
    Ok(source_files
        .into_iter()
        .map(|file| look_up_info(file, Path::new(&destination), &dest_files, &mut indices))
        .log_errors_and_collect())
}

/// Looks up the destination mapping for a source image.
/// if it already exists in the output directory, that filename will be returned.
/// Otherwise, it will be assigned the next available index.
fn look_up_info<'a>(
    input: &'a Path,
    destination: &Path,
    destination_files: &[String],
    indices: &mut dyn Iterator<Item = u32>,
) -> Result<ImageInfo<'a>> {
    let file_stem = input
        .file_stem()
        .and_then(|f| f.to_str())
        .ok_or_else(|| anyhow!("Failed to get filename for {:?}", input))?;

    // If the file is foo.png, we're looking for an existing file named NNNN-foo.bin, where
    // N is a decimal digit. If it exists, we'll overwrite it, otherwise we'll use the next index.
    let re = Regex::new(&format!(r"^\d+-{}.bin$", escape(file_stem)))
        .with_context(|| format!("Failed to construct regex for {:?}", input))?;

    let existing_name = destination_files.iter().find(|x| re.is_match(x));

    let output = destination.join(existing_name.map(PathBuf::from).unwrap_or_else(|| {
        PathBuf::from(format!(
            "{:04}-{}.bin",
            indices.next().expect("Ran out of indices"),
            file_stem
        ))
    }));

    Ok(ImageInfo { input, output })
}
