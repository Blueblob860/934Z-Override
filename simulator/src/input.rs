use gilrs::{GamepadId, Gilrs};
use roboscope_ipc::{SimServices, controller::{ControllerInput, ControllerStatus}, display::DisplayInput};

use crate::{ContPubType, DispInputPubType};

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

pub struct V5InputHandler {
    disp_pub: DispInputPubType,
    mouse_coords: egui::Vec2 = egui::Vec2 { x: 0.0, y: 0.0 },
    pointer_down: bool = false,
    presses: u32 = 0,
    releases: u32 = 0,

    cont1_pub: ContPubType,
    cont2_pub: ContPubType,
    gilrs: Gilrs,
    gp1: Option<GamepadId> = None,
    gp2: Option<GamepadId> = None,
    gp1_state: ControllerInput = DEF_CONT_INPUT,
    gp2_state: ControllerInput = DEF_CONT_INPUT
}

impl V5InputHandler {
    pub fn new(sim: &SimServices) -> anyhow::Result<Self> {
        let disp_pub = sim.display_input()?.publisher_builder().create()?;
        let cont1_pub = sim.primary_controller_input()?.publisher_builder().create()?;
        let cont2_pub = sim.secondary_controller_input()?.publisher_builder().create()?;
        Ok(Self {
            disp_pub,
            cont1_pub,
            cont2_pub,
            gilrs: Gilrs::new().unwrap(),
            ..
        })
    }

    pub fn update_controllers(&mut self) {
        // Only send an updated state if there was a change
        let mut cont1_updated = false;
        let mut cont2_updated = false;

        // Loop through all the gamepad updates since the last frame
        while let Some(gilrs::Event { id, event, .. }) = self.gilrs.next_event() {
            // If there's a new gamepad connection,
            // and one of the two controllers are missing,
            // set that gamepad as the missing controller
            if self.gp1.is_none() { self.gp1 = Some(id); }
            else if self.gp2.is_none() { self.gp2 = Some(id); }

            // If either of the two gamepads received an
            // update this event, update their state
            if id == self.gp1.unwrap() {
                self.gp1_state = handle_gamepad_event(self.gp1_state, event);
                cont1_updated = true;
            } else if id == self.gp2.unwrap() {
                self.gp2_state = handle_gamepad_event(self.gp2_state, event);
                cont2_updated = true;
            }
        }

        // If one of the two controllers recieved an update then
        // pass it along to the brain
        if cont1_updated {
            let _ = self.cont1_pub.send_copy(self.gp1_state);
        }
        if cont2_updated {
            let _ = self.cont2_pub.send_copy(self.gp2_state);
        }
    }

    pub fn update_touch(&mut self, ui: &mut egui::Ui) {
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
            let _ = self.disp_pub.send_copy(DisplayInput {
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
    }
}

fn handle_gamepad_event(mut state: ControllerInput, event: gilrs::EventType) -> ControllerInput {
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