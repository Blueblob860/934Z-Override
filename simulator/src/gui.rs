use roboscope_ipc::display::DisplayFrame;
use serde::{Deserialize, Serialize};

use crate::{FrameSubType, input::V5InputHandler};

#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ViewportUi {
    #[serde(skip)]
    frames: Option<FrameSubType> = None,
    #[serde(skip)]
    last_frame: Option<Box<DisplayFrame>> = None,
    #[serde(skip)]
    texture: Option<egui::TextureHandle> = None,
    #[serde(skip)]
    input: Option<V5InputHandler> = None,
}

impl ViewportUi {
    pub fn new(cc: &eframe::CreationContext, input: V5InputHandler, frames: FrameSubType) -> Self {
        let mut stored: ViewportUi = if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Default::default()
        };
        stored.input = Some(input);
        stored.frames = Some(frames);
        stored.texture = Some(cc.egui_ctx.load_texture("displaytex", egui::ColorImage::example(), egui::TextureOptions::NEAREST));
        stored
    }

    pub fn fetch_frame(&mut self) {
        if let Some(frame) = self.frames.as_ref().unwrap().receive().expect("Should receive frame") {
            self.last_frame = Some(Box::new(frame.clone()));
        }
    }
}

impl eframe::App for ViewportUi {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if let Some(input) = self.input.as_mut() {
            input.update_controllers();
        }

        egui::Panel::top("top_panel").show(ui, |ui| {
            egui::MenuBar::new().ui(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Quit").clicked() {
                        ui.send_viewport_cmd(egui::ViewportCommand::Close);
                    }
                });
                ui.add_space(16.0);
                egui::widgets::global_theme_preference_buttons(ui);
            });
        });

        egui::CentralPanel::default().show(ui, |ui| {
            egui::Window::new("Display Viewer")
                .resizable(false)
                .auto_sized()
                .frame(egui::Frame::new().inner_margin(0.0).corner_radius(4.0))
                .show(ui, |ui| {
                    if let Some(input) = self.input.as_mut() {
                        input.update_touch(ui);
                    }
                    
                    self.fetch_frame();
                    if self.last_frame.is_none() { return; }
                    let pix_vec: Vec<u32> = self.last_frame.as_ref().unwrap().buffer.into();
                    self.texture.as_mut().unwrap().set(
                        egui::ColorImage {
                            size: [480, 272],
                            source_size: [480.0, 272.0].into(),
                            pixels: pix_vec.iter().cloned().map(|v| egui::Color32::from_rgb(((v & 0xFF0000) >> 16) as u8, ((v & 0x00FF00) >> 8) as u8, (v & 0x0000FF) as u8)).collect()
                        },
                        egui::TextureOptions::NEAREST
                    );
                    let size = self.texture.as_mut().unwrap().size_vec2();
                    let sized_tex = egui::load::SizedTexture::new(self.texture.as_mut().unwrap(), size);

                    ui.add(egui::Image::new(sized_tex).fit_to_exact_size(size));
                });
        });
        ui.ctx().request_repaint_after_secs(0.05);
    }
}