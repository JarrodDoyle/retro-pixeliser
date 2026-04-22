use image::{ImageError, ImageReader, RgbImage, imageops::FilterType};
use palette::{IntoColor, Oklab, Srgb, cast::FromComponents, color_difference::EuclideanDistance};

const PALETTE_PATH: &str = "./res/palette.png";
const INPUT_PATH: &str = "./res/input.png";
const OUTPUT_PATH: &str = "./res/output.png";
const DOWNSCALE: u32 = 4;

fn palette_from_image(image: &RgbImage) -> Vec<Oklab> {
    let mut colours: Vec<Oklab> = vec![];
    for pixel in <&[Srgb<u8>]>::from_components(&**image) {
        colours.push(pixel.into_linear().into_color());
    }
    colours
}

fn apply_palette(image: &mut RgbImage, palette: &Vec<Oklab>) {
    for pixel in <&mut [Srgb<u8>]>::from_components(&mut **image) {
        let pixel_colour = pixel.into_linear().into_color();
        let closest_colour = get_closest_palette_colour(palette, pixel_colour);
        *pixel = Srgb::from_linear(closest_colour.into_color());
    }
}

fn get_closest_palette_colour(palette: &Vec<Oklab>, colour: Oklab) -> Oklab {
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
