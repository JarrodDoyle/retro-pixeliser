use image::{ImageError, ImageReader, imageops::FilterType};

const INPUT_PATH: &str = "./res/input.png";
const OUTPUT_PATH: &str = "./res/output.png";
const DOWNSCALE: u32 = 4;

fn main() -> Result<(), ImageError> {
    let image = ImageReader::open(INPUT_PATH)?.decode()?;
    let output_image = image.resize(
        image.width() / DOWNSCALE,
        image.height() / DOWNSCALE,
        FilterType::Nearest,
    );

    output_image.save(OUTPUT_PATH)?;
    Ok(())
}
