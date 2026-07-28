use gilrs::{GamepadId, Gilrs};
use roboscope_ipc::{Publisher, Subscriber, controller::{ControllerInput, ControllerStatus}, display::{DisplayFrame, DisplayInput}};
use serde::{Deserialize, Serialize};

type InputPubType = Publisher<DisplayInput>;
type FrameSubType = Subscriber<DisplayFrame>;
type ContPubType = Publisher<ControllerInput>;

const DEF_CONT_INPUT: ControllerInput = ControllerInput {
    connected: ControllerStatus::Offline,
    left_x: 0, left_y: 0,
    right_x: 0, right_y: 0,
    button_l1: false, button_l2: false,
    button_r1: false, button_r2: false,
    button_up: false, button_down: false,
    button_left: false, button_right: false,
    button_x: false, button_b: false,
    button_y: false, button_a: false,
    button_power: false,
};

#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct ViewportUi {
    #[serde(skip)]
    input: Option<InputPubType> = None,
    #[serde(skip)]
    frames: Option<FrameSubType> = None,
    #[serde(skip)]
    cont1: Option<ContPubType> = None,
    #[serde(skip)]
    last_frame: Option<Box<DisplayFrame>> = None,
    #[serde(skip)]
    texture: Option<egui::TextureHandle> = None,
    #[serde(skip)]
    gilrs: Option<Gilrs> = None,
    #[serde(skip)]
    primary_gp: Option<GamepadId> = None,
    #[serde(skip)]
    primary_gp_state: ControllerInput = DEF_CONT_INPUT,
    mouse_coords: egui::Vec2,
    presses: u32 = 0,
    releases: u32 = 0,
    pointer_down: bool = false,
}

impl ViewportUi {
    pub fn new(cc: &eframe::CreationContext, input: InputPubType, frames: FrameSubType, cont1_in: ContPubType) -> Self {
        let mut stored: ViewportUi = if let Some(storage) = cc.storage {
            eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default()
        } else {
            Default::default()
        };
        stored.input = Some(input);
        stored.frames = Some(frames);
        stored.cont1 = Some(cont1_in);
        stored.texture = Some(cc.egui_ctx.load_texture("displaytex", egui::ColorImage::example(), egui::TextureOptions::NEAREST));
        stored.gilrs = Some(Gilrs::new().unwrap());
        stored
    }

    pub fn fetch_frame(&mut self) {
        if let Some(frame) = self.frames.as_ref().unwrap().receive().expect("Should receive frame") {
            self.last_frame = Some(Box::new(frame.clone()));
        }
    }
}

fn handle_gamepad_event(mut state: ControllerInput, event: gilrs::EventType) -> ControllerInput{
    match event {
        gilrs::EventType::ButtonPressed(button, _) => {
            match button {
                gilrs::Button::South => { state.button_b = true; },
                gilrs::Button::East => { state.button_a = true; },
                gilrs::Button::North => { state.button_x = true; },
                gilrs::Button::West => { state.button_y = true; },
                gilrs::Button::LeftTrigger => { state.button_l1 = true; },
                gilrs::Button::LeftTrigger2 => { state.button_l2 = true; },
                gilrs::Button::RightTrigger => { state.button_r1 = true; },
                gilrs::Button::RightTrigger2 => { state.button_r2 = true; },
                gilrs::Button::Start => { state.button_power = true; },
                gilrs::Button::DPadUp => { state.button_up = true; },
                gilrs::Button::DPadDown => { state.button_down = true; },
                gilrs::Button::DPadLeft => { state.button_left = true; },
                gilrs::Button::DPadRight => { state.button_right = true; },
                _ => {},
            }
        },
        gilrs::EventType::ButtonReleased(button, _) => {
            match button {
                gilrs::Button::South => { state.button_b = false; },
                gilrs::Button::East => { state.button_a = false; },
                gilrs::Button::North => { state.button_x = false; },
                gilrs::Button::West => { state.button_y = false; },
                gilrs::Button::LeftTrigger => { state.button_l1 = false; },
                gilrs::Button::LeftTrigger2 => { state.button_l2 = false; },
                gilrs::Button::RightTrigger => { state.button_r1 = false; },
                gilrs::Button::RightTrigger2 => { state.button_r2 = false; },
                gilrs::Button::Start => { state.button_power = false; },
                gilrs::Button::DPadUp => { state.button_up = false; },
                gilrs::Button::DPadDown => { state.button_down = false; },
                gilrs::Button::DPadLeft => { state.button_left = false; },
                gilrs::Button::DPadRight => { state.button_right = false; },
                _ => {},
            }
        },
        gilrs::EventType::AxisChanged(axis, v, _) => {
            match axis {
                gilrs::Axis::LeftStickX => { state.left_x = (v * 127.0) as i32; },
                gilrs::Axis::LeftStickY => { state.left_y = (v * 127.0) as i32; },
                gilrs::Axis::RightStickX => { state.right_x = (v * 127.0) as i32; },
                gilrs::Axis::RightStickY => { state.right_y = (v * 127.0) as i32; },
                gilrs::Axis::DPadX => { 
                    if v < -0.5 { state.button_left = true; state.button_right = false; }
                    else if v > 0.5 { state.button_left = false; state.button_right = true; }
                    else { state.button_left = false; state.button_right = false; }
                },
                gilrs::Axis::DPadY => { 
                    if v < -0.5 { state.button_down = true; state.button_up = false; }
                    else if v > 0.5 { state.button_down = false; state.button_up = true; }
                    else { state.button_down = false; state.button_up = false; }
                },
                _ => {},
            }
        },
        gilrs::EventType::Connected => {
            state.connected = ControllerStatus::Wireless;
        },
        gilrs::EventType::Disconnected => {
            state.connected = ControllerStatus::Offline;
        },
        _ => {},
    }
    state
}

impl eframe::App for ViewportUi {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if let Some(gilrs) = &mut self.gilrs {
            let mut controller_updated = false;
            while let Some(gilrs::Event { id, event, .. }) = gilrs.next_event() {
                if self.primary_gp.is_none() { self.primary_gp = Some(id); }
                if id == self.primary_gp.unwrap() {
                    self.primary_gp_state = handle_gamepad_event(self.primary_gp_state, event);
                }
                // println!("New event from {id}: {event:?}");
                controller_updated = true;
            }
            if let Some(cont1) = self.cont1.as_mut() && controller_updated {
                let _ = cont1.send_copy(self.primary_gp_state);
            }
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
                //.frame(egui::Frame::new().inner_margin(2.0).corner_radius(4.0))
                .show(ui, |ui| {
                    let widg_pos = ui.next_widget_position();
                    let (clicked, released, held, moved) = ui.input(|i| {
                        let interact_in_bounds = if let Some(pos) = i.pointer.press_origin() {
                            let adj_mp = pos - widg_pos;
                            adj_mp.x >= 0.0 && adj_mp.y >= 32.0 && adj_mp.x <= 480.0 && adj_mp.y <= 272.0
                        } else { false };
                        let hold = i.pointer.primary_down() && interact_in_bounds;
                        let click = !self.pointer_down && hold;
                        let release = self.pointer_down && !hold;
                        self.pointer_down = hold;
                        let moved = if let Some(pos) = i.pointer.latest_pos() && hold {
                            let adj_mp = pos - widg_pos;
                            let moved = (self.mouse_coords - adj_mp).length_sq() > 1.0;
                            self.mouse_coords = adj_mp;
                            moved
                        } else { false };
                        if click { self.presses = self.presses.wrapping_add(1); }
                        if release { self.releases = self.releases.wrapping_add(1); }
                        (click, release, hold, moved)
                    });

                    if clicked || released || (held && moved) {
                        let _ = self.input.as_mut().unwrap().send_copy(DisplayInput {
                            kind: if clicked {
                                roboscope_ipc::display::DisplayInputKind::Press
                            } else if !clicked && held {
                                roboscope_ipc::display::DisplayInputKind::Hold
                            } else {
                                roboscope_ipc::display::DisplayInputKind::Release
                            },
                            press_count: self.presses,
                            release_count: self.releases,
                            x: (self.mouse_coords.x) as i16,
                            y: (self.mouse_coords.y) as i16,
                        });
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
                    ui.label(format!("left: {}, {}", self.primary_gp_state.left_x, self.primary_gp_state.left_y));
                    ui.label(format!("right: {}, {}", self.primary_gp_state.right_x, self.primary_gp_state.right_y));
                });
        });
        ui.ctx().request_repaint_after_secs(0.05);
    }
}