use ::image::ImageError;

mod cli;
mod image;

fn main() -> Result<(), ImageError> {
    cli::run()
}
