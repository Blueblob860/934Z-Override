#![feature(default_field_values)]

use roboscope_ipc::{Publisher, SimServices, Subscriber, controller::ControllerInput, display::{DisplayFrame, DisplayInput}};

use crate::input::V5InputHandler;

pub mod gui;
pub mod input;

pub type DispInputPubType = Publisher<DisplayInput>;
pub type FrameSubType = Subscriber<DisplayFrame>;
pub type ContPubType = Publisher<ControllerInput>;

fn main() -> anyhow::Result<()> {
    let sim = SimServices::join(Some("Bloblib"), &roboscope_ipc::Config::default())?;
    //let mut readings = DeviceReadings::default();
    //let device_cmds = sim.device_cmds()?.subscriber_builder().create()?;
    let display_sub = sim.display_frames()?.subscriber_builder().create()?;
    let input = V5InputHandler::new(&sim)?;

    gui_init(display_sub, input).unwrap();
    Ok(())
}

fn gui_init(disp_output: FrameSubType, input: V5InputHandler) -> eframe::Result {
    let native_opts = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([834.0, 480.0])
            .with_max_inner_size([300.0, 220.0])
            .with_title("Bloblib Simulator"),
        ..Default::default()
    };

    eframe::run_native(
        "Bloblib Simulator", 
        native_opts,
        Box::new(|cc| Ok(Box::new(gui::ViewportUi::new(cc, input, disp_output)))) 
    )
}