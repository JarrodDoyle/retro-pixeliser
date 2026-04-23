use std::{ffi::OsStr, fs, path::PathBuf};

use clap::Parser;
use image::{ImageError, ImageFormat, ImageReader, RgbImage, imageops::FilterType};
use itertools::Itertools;
use palette::{IntoColor, Oklab, Srgb, cast::FromComponents, color_difference::EuclideanDistance};
use rayon::prelude::*;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    palette_path: PathBuf,
    #[arg(short, long)]
    input_path: PathBuf,
    #[arg(short, long)]
    output_path: Option<PathBuf>,
    #[arg(short = 's', long, default_value_t = 4)]
    pixel_scale: u32,
    #[arg(short, long)]
    dither: bool,
    #[arg(short = 'e', long, default_value_t = 2)]
    dither_exponent: u32,
    #[arg(short = 't', long, default_value_t = 0.05)]
    dither_threshold: f32,
    #[arg(short, long)]
    batch: bool,
}

// Modified slightly from https://nelari.us/post/quick_and_dirty_dithering/#bayer-matrix
struct BayerMatrix {
    size: u32,
    matrix: Vec<u8>,
}

impl BayerMatrix {
    fn new(exponent: u32) -> Self {
        let size = 2_u32.pow(exponent);
        let matrix = (0..size)
            .cartesian_product(0..size)
            .map(|(x, y)| {
                let xc = x ^ y;
                let yc = y;
                let mut v = 0;
                for p in (0..exponent).rev() {
                    let bit_idx = 2 * (exponent - p - 1);
                    v |= ((yc >> p) & 1) << bit_idx;
                    v |= ((xc >> p) & 1) << (bit_idx + 1);
                }
                v as f32 as u8
            })
            .collect();
        Self { size, matrix }
    }

    fn index(&self, x: u32, y: u32) -> u8 {
        let j = x % self.size;
        let i = y % self.size;
        let idx = (i * self.size + j) as usize;
        self.matrix[idx]
    }
}

fn palette_from_image(image: &RgbImage) -> Vec<Oklab> {
    let mut colours: Vec<Oklab> = vec![];
    for pixel in <&[Srgb<u8>]>::from_components(&**image) {
        colours.push(pixel.into_linear().into_color());
    }
    colours
}

fn apply_palette(pixel_colour: &mut Oklab, palette: &Vec<Oklab>) {
    *pixel_colour = get_closest_palette_colour(palette, pixel_colour);
}

// Pattern dithering: https://bisqwit.iki.fi/story/howto/dither/jy/#PatternDitheringThePatentedAlgorithmUsedInAdobePhotoshop
fn apply_palette_dithered(
    x: u32,
    y: u32,
    pixel_colour: &mut Oklab,
    palette: &Vec<Oklab>,
    bayer_matrix: &BayerMatrix,
    threshold: f32,
) {
    let mut candidates: Vec<Oklab> = vec![];
    let mut error = Oklab::new(0.0, 0.0, 0.0);
    let matrix_element_count = bayer_matrix.size.pow(2);
    for _ in 0..matrix_element_count {
        let sample = *pixel_colour + error * threshold;
        let candidate = get_closest_palette_colour(palette, &sample);
        candidates.push(candidate);
        error += *pixel_colour - candidate;
    }

    candidates.sort_by(|Oklab { l: l1, .. }, Oklab { l: l2, .. }| l1.partial_cmp(l2).unwrap());
    *pixel_colour = candidates[bayer_matrix.index(x, y) as usize];
}

fn get_closest_palette_colour(palette: &Vec<Oklab>, colour: &Oklab) -> Oklab {
    let mut closest_colour = Oklab::new(0.0, 0.0, 0.0);
    let mut closest_distance_squared = f32::MAX;
    for palette_colour in palette {
        let distance = colour.distance_squared(*palette_colour);
        if distance < closest_distance_squared {
            closest_distance_squared = distance;
            closest_colour = *palette_colour
        }
    }
    closest_colour
}

fn main() -> Result<(), ImageError> {
    let args = Args::parse();

    let mut paths = vec![];
    if args.batch {
        // TODO: Handle input and output not being dirs!
        let output_dir = match args.output_path {
            Some(path) if path.is_dir() => path,
            _ => args.input_path.join("output"),
        };

        let input_paths = fs::read_dir(&args.input_path)?
            .filter_map(|e| e.ok())
            .filter_map(|e| match e.path().extension() {
                Some(ext) if ImageFormat::from_extension(ext).is_some() => Some(e.path()),
                _ => None,
            });
        for input_path in input_paths {
            let output_path = output_dir.join(input_path.file_name().unwrap_or_default());
            paths.push((input_path, output_path));
        }

        if !output_dir.exists() {
            fs::create_dir(output_dir)?;
        }
    } else {
        let output_path = match args.output_path {
            Some(path) => path,
            None => {
                let mut file_name = args.input_path.file_prefix().unwrap_or_default().to_owned();
                file_name.push(OsStr::new("_output"));
                let mut path = args.input_path.with_file_name(file_name);
                if let Some(ext) = args.input_path.extension() {
                    path.set_extension(ext);
                }
                path
            }
        };
        paths.push((args.input_path, output_path));
    }

    let palette_image = ImageReader::open(args.palette_path)?.decode()?.into_rgb8();
    let palette = palette_from_image(&palette_image);
    let bayer_matrix = BayerMatrix::new(args.dither_exponent);
    for (input_path, output_path) in paths {
        let image = ImageReader::open(input_path)?.decode()?;
        let image = image.resize(
            image.width() / args.pixel_scale,
            image.height() / args.pixel_scale,
            FilterType::Nearest,
        );
        let mut output_image = image.into_rgb8();

        output_image
            .par_enumerate_pixels_mut()
            .for_each(|(x, y, pixel)| {
                let mut pixel_colour: Oklab = Srgb::from(pixel.0).into_linear().into_color();

                if args.dither {
                    apply_palette_dithered(
                        x,
                        y,
                        &mut pixel_colour,
                        &palette,
                        &bayer_matrix,
                        args.dither_threshold,
                    );
                } else {
                    apply_palette(&mut pixel_colour, &palette);
                }

                let srgb_colour = Srgb::from_linear(pixel_colour.into_color());
                *pixel = image::Rgb([srgb_colour.red, srgb_colour.green, srgb_colour.blue]);
            });

        output_image.save(output_path)?;
    }

    Ok(())
}
