use image::{ImageError, ImageReader, RgbImage, imageops::FilterType};
use palette::{Srgb, cast::FromComponents, color_difference::EuclideanDistance};

const PALETTE_PATH: &str = "./res/palette.png";
const INPUT_PATH: &str = "./res/input.png";
const OUTPUT_PATH: &str = "./res/output.png";
const DOWNSCALE: u32 = 4;

fn palette_from_image(image: &RgbImage) -> Vec<Srgb> {
    let mut colours: Vec<Srgb> = vec![];
    for pixel in <&[Srgb<u8>]>::from_components(&**image) {
        colours.push(pixel.into_format());
    }
    colours
}

fn apply_palette(image: &mut RgbImage, palette: &Vec<Srgb>) {
    for pixel in <&mut [Srgb<u8>]>::from_components(&mut **image) {
        let pixel_srgb: Srgb = pixel.into_format();
        let closest_colour = get_closest_palette_colour(palette, pixel_srgb);
        *pixel = closest_colour.into_format();
    }
}

fn get_closest_palette_colour(palette: &Vec<Srgb>, colour: Srgb) -> Srgb {
    let mut closest_colour = Srgb::new(0.0, 0.0, 0.0);
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
    let image = ImageReader::open(INPUT_PATH)?.decode()?;
    let image = image.resize(
        image.width() / DOWNSCALE,
        image.height() / DOWNSCALE,
        FilterType::Nearest,
    );

    let palette_rgb = palette_from_image(&ImageReader::open(PALETTE_PATH)?.decode()?.into_rgb8());
    let mut output_image = image.into_rgb8();
    apply_palette(&mut output_image, &palette_rgb);

    output_image.save(OUTPUT_PATH)?;
    Ok(())
}
