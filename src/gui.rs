use std::sync::Arc;

use anyhow::Result;
use eframe::egui::{self, ImageData, TextureOptions};
use image::{EncodableLayout, RgbImage};
use palette::LinSrgb;

use crate::image::{ImageSettings, apply_effects, load_image, palette_from_image};

#[derive(Default)]
struct GuiApp {
    base_image: Option<RgbImage>,
    output_image: Option<RgbImage>,
    palette: Vec<LinSrgb>,
    texture: Option<egui::TextureHandle>,
    image_settings: ImageSettings,
}

impl GuiApp {
    fn update_texture(&mut self, ctx: &egui::Context) {
        if let Some(image) = &self.base_image {
            self.output_image = Some(apply_effects(image, &self.palette, &self.image_settings));
        }

        if let Some(image) = &self.output_image {
            let egui_image = Arc::new(egui::ColorImage::from_rgb(
                [image.width() as usize, image.height() as usize],
                image.as_bytes(),
            ));
            if let Some(texture) = &mut self.texture {
                texture.set(ImageData::Color(egui_image), TextureOptions::NEAREST);
            } else {
                self.texture = Some(ctx.load_texture("image", egui_image, Default::default()));
            }
        }
    }
}

impl eframe::App for GuiApp {
    fn ui(&mut self, ui: &mut eframe::egui::Ui, _frame: &mut eframe::Frame) {
        let mut new_settings = self.image_settings;

        egui::Panel::right("right_panel").show_inside(ui, |ui| {
            ui.vertical(|ui| {
                if ui.button("Open File...").clicked()
                    && let Some(path) = rfd::FileDialog::new().pick_file()
                    && let Ok(image) = load_image(&path)
                {
                    self.base_image = Some(image);
                    self.update_texture(ui.ctx());
                }

                if ui.button("Select Palette...").clicked()
                    && let Some(path) = rfd::FileDialog::new().pick_file()
                    && let Ok(image) = load_image(&path)
                {
                    self.palette = palette_from_image(&image);
                    self.update_texture(ui.ctx());
                }

                if ui.button("Save As...").clicked()
                    && let Some(image) = &self.output_image
                    && let Some(path) = rfd::FileDialog::new()
                        .add_filter("PNG", &["png"])
                        .save_file()
                {
                    let _ = image.save(path);
                }

                ui.add(egui::Slider::new(&mut new_settings.scale, 1..=16).text("Scale"));
                ui.add(egui::Slider::new(&mut new_settings.hue, 0..=360).text("Hue"));
                ui.add(
                    egui::Slider::new(&mut new_settings.saturation, -100..=100).text("Saturation"),
                );
                ui.add(
                    egui::Slider::new(&mut new_settings.brightness, -100..=100).text("Brightness"),
                );
                ui.add(egui::Slider::new(&mut new_settings.contrast, -100..=100).text("contrast"));
                ui.group(|ui| {
                    ui.checkbox(&mut new_settings.dither, "Dither");
                    if !new_settings.dither {
                        ui.disable();
                    }

                    ui.add(
                        egui::Slider::new(&mut new_settings.dither_exponent, 1..=3)
                            .text("Dither exponent"),
                    );
                    ui.add(
                        egui::Slider::new(&mut new_settings.dither_threshold, 0.0..=1.0)
                            .text("Dither threshold"),
                    );
                });
            });
        });

        if new_settings != self.image_settings {
            self.image_settings = new_settings;
            self.update_texture(ui.ctx());
        }

        egui::CentralPanel::default().show_inside(ui, |ui| {
            if let Some(texture) = &self.texture {
                // I wanted to use `maintain_aspect_ratio` but it wasn't doing anything :)
                ui.centered_and_justified(|ui| {
                    ui.image((
                        texture.id(),
                        texture.size_vec2()
                            * (ui.available_size() / texture.size_vec2()).min_elem(),
                    ))
                });
            }
        });
    }
}

pub fn run() -> Result<()> {
    let options = eframe::NativeOptions {
        renderer: eframe::Renderer::Wgpu,
        ..Default::default()
    };
    eframe::run_native(
        "RetroPixel",
        options,
        Box::new(|_cc| Ok(Box::<GuiApp>::default())),
    )?;

    Ok(())
}
