#![feature(default_field_values)]

use std::thread::spawn;

use roboscope_ipc::{Publisher, SimServices, Subscriber, display::{DisplayFrame, DisplayInput}};

pub mod gui;

fn main() -> anyhow::Result<()> {
    let sim = SimServices::join(Some("Bloblib"), &roboscope_ipc::Config::default())?;
    //let mut readings = DeviceReadings::default();
    //let device_cmds = sim.device_cmds()?.subscriber_builder().create()?;
    let display_pub = sim.display_input()?.publisher_builder().create()?;
    let display_sub = sim.display_frames()?.subscriber_builder().create()?;
    gui_init(display_pub, display_sub).unwrap();
    Ok(())
}

fn gui_init(input: Publisher<DisplayInput>, frames: Subscriber<DisplayFrame>) -> eframe::Result {
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
        Box::new(|cc| Ok(Box::new(gui::ViewportUi::new(cc, input, frames)))) 
    )
}