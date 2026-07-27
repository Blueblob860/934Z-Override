use roboscope_ipc::{Publisher, Subscriber, display::{DisplayFrame, DisplayInput}};
use serde::{Deserialize, Serialize};

type InputSubType = Publisher<DisplayInput>;
type FrameSubType = Subscriber<DisplayFrame>;

#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ViewportUi {
    #[serde(skip)]
    input: Option<Publisher<DisplayInput>> = None,
    #[serde(skip)]
    frames: Option<Subscriber<DisplayFrame>> = None,
    #[serde(skip)]
    last_frame: Option<Box<DisplayFrame>>,
    #[serde(skip)]
    texture: Option<egui::TextureHandle>,
    mouse_coords: (f32, f32) = (0.0, 0.0),
    clicks: u32 = 0,
    pointer_down: bool = false,
}

impl ViewportUi {
    pub fn new(cc: &eframe::CreationContext, input: InputSubType, frames: FrameSubType) -> Self {
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
                .frame(egui::Frame::new().inner_margin(2.0).corner_radius(4.0))
                .show(ui, |ui| {
                    let widg_pos = ui.next_widget_position().to_vec2();
                    let (hold, click, release_count) = ui.input(|i| {
                        let interact_in_bounds = if let Some(pos) = i.pointer.press_origin() {
                            let adj_mp = pos.to_vec2() - widg_pos;
                            adj_mp.x >= 0.0 && adj_mp.y >= 32.0 && adj_mp.x <= 480.0 && adj_mp.y <= 272.0
                        } else { false };
                        let hold = i.pointer.primary_down() && interact_in_bounds;
                        let click = !self.pointer_down && hold;
                        self.pointer_down = hold;
                        if let Some(pos) = i.pointer.latest_pos() && hold {
                            let adj_mp = pos.to_vec2() - widg_pos;
                            self.mouse_coords = (adj_mp.x, adj_mp.y);
                        }
                        let release_count = self.clicks;
                        if click { self.clicks = self.clicks.wrapping_add(1); }
                        (hold, click, release_count)
                    });

                    if let Err(e) = self.input.as_mut().unwrap().send_copy(DisplayInput {
                        kind: if click {
                            roboscope_ipc::display::DisplayInputKind::Press
                        } else if !click && hold {
                            roboscope_ipc::display::DisplayInputKind::Hold
                        } else {
                            roboscope_ipc::display::DisplayInputKind::Release
                        },
                        press_count: self.clicks,
                        release_count,
                        x: (self.mouse_coords.0) as i16,
                        y: (self.mouse_coords.1) as i16,
                    }) {
                        eprintln!("An error occured while sending a DisplayInput: {}", e);
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
                    ui.label(format!("{}, {}", hold, click));
                    ui.label(format!("{}, {}", self.mouse_coords.0, self.mouse_coords.1));
                });
        });
        ui.ctx().request_repaint_after_secs(0.05);
    }
}