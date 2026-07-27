#![feature(default_field_values)]

use std::time::{Duration, Instant};

use bloblib::prelude::*;
use vexide::{color::Color, display::{Font, FontSize, Rect, Text, TouchState}, prelude::*};

#[cfg(target_os = "vexos")]
use vex_sdk_jumptable as _;

#[cfg(not(target_os = "vexos"))]
#[allow(unused_imports)]
use vex_sdk_desktop::sdk as _;

struct Robot {
    chassis: Chassis,
    display: Display
}

impl Compete for Robot {
    async fn autonomous(&mut self) {
        println!("Autonomous!");
        let mut timer = Instant::now();
        self.chassis.move_to_pose(12.0, 12.0, 0.0, 5000.0, MoveToPoseParams { local: true, .. }).await;
        self.chassis.move_to_pose(0.0, 0.0, 0.0, 5000.0, MoveToPoseParams { local: true, forwards: false, .. }).await;
        println!("Move to Pose roundtrip completed in {}s", timer.elapsed().as_secs_f64());
        sleep(Duration::from_secs(3)).await;
        timer = Instant::now();
        self.chassis.move_to_point(12.0, 0.0, 5000.0, MoveToPointParams { local: true, .. }).await;
        self.chassis.move_to_point(0.0, 0.0, 5000.0, MoveToPointParams { local: true, forwards: false, .. }).await;
        println!("Move to Point roundtrip completed in {}s", timer.elapsed().as_secs_f64());
        sleep(Duration::from_secs(3)).await;
        timer = Instant::now();
        self.chassis.turn_to_heading(90.0, 5000.0, TurnToHeadingParams { .. }).await;
        self.chassis.turn_to_heading(0.0, 5000.0, TurnToHeadingParams { .. }).await;
        println!("Turn to Heading roundtrip completed in {}s", timer.elapsed().as_secs_f64());
        sleep(Duration::from_secs(3)).await;
        timer = Instant::now();
        self.chassis.turn_to_point(0.0, 10.0, 5000.0, TurnToPointParams { .. }).await;
        self.chassis.turn_to_point(0.0, 0.0, 5000.0, TurnToPointParams { .. }).await;
        println!("Turn to Point roundtrip completed in {}s", timer.elapsed().as_secs_f64());
    }

    async fn driver(&mut self) {
        println!("Driver!");
        self.display.set_render_mode(vexide::display::RenderMode::DoubleBuffered);
        loop {
            // let controller_state = self.chassis.controller.write().await.state();
            // if controller_state.is_err() {
            //     sleep(Duration::from_millis(25)).await;
            //     continue;
            // }
            // let controller_state = controller_state.unwrap();
            // let left_y = controller_state.left_stick.y();
            // let right_x = controller_state.right_stick.x();
            // self.chassis.arcade(left_y, right_x, false, 0.5).await;
            let touch = self.display.touch_status();
            println!("{}, {}", touch.point.x, touch.point.y);
            //self.display.erase(Color::BLACK);
            //self.display.draw_text(
            //    &Text::from_string(format!("touch_pos: {}, {}", touch.point.x, touch.point.y),
            //    Font::new(FontSize::SMALL, vexide::display::FontFamily::Proportional),
            //    [10, 10]), Color::WHITE, Some(Color::BLACK));
            //self.display.draw_text(
            //    &Text::from_string(format!("click: {}, {}", touch.state == TouchState::Pressed, touch.state == TouchState::Held),
            //    Font::new(FontSize::SMALL, vexide::display::FontFamily::Proportional),
            //    [10, 24]), Color::WHITE, Some(Color::BLACK));
            // self.display.fill(&Rect::new([0, 0], [480, 240]), 0x006fff);
            //self.display.render();
            sleep(Duration::from_millis(500)).await;
        }
    }

    async fn disabled(&mut self) {
        self.chassis.cancel_all_motions().await;
    }
}

#[vexide::main]
async fn main(peripherals: Peripherals) {
    #[cfg(not(target_os = "vexos"))]
    {
        use tracing::level_filters::LevelFilter;
 
        tracing_subscriber::fmt()
            .with_max_level(LevelFilter::WARN)
            .init();
        vex_sdk_desktop::init().expect("Simulator didn't initialize");
    }

    // Drivetrain struct
    let drive = Drivetrain::new(
        // Left Side Motors
        vec![
            FilteredMotor::new(peripherals.port_12, Gearset::Blue, Direction::Forward),
            FilteredMotor::new(peripherals.port_13, Gearset::Blue, Direction::Reverse),
            FilteredMotor::new(peripherals.port_14, Gearset::Blue, Direction::Reverse)
        ],
        // Right Side Motors
        vec![
            FilteredMotor::new(peripherals.port_17, Gearset::Blue, Direction::Forward),
            FilteredMotor::new(peripherals.port_18, Gearset::Blue, Direction::Forward),
            FilteredMotor::new(peripherals.port_19, Gearset::Blue, Direction::Reverse)
        ],
        10.6, // Track Width
        2.75, // Wheel Size
        36.0/45.0, // Gear Ratio (600 -> 450)
        8.0 // Horizontal Drift, 8.0 for tractions, 2.0 for omnis
    );

    // Odom Sensors, no tracking wheels yet :(
    let sensors = Sensors {
        imu: Some(InertialSensor::new(peripherals.port_20)), // Inertial Sensor
        ..Default::default()
    };

    // Main Chassis object, start it off with drivetrain, sensors and controller from earlier
    let mut chassis = Chassis::new(drive, sensors, peripherals.primary_controller);
    
    // Set linear PID
    chassis.linear = PidBuilder { kp: 2.0, kd: 8.0, .. }.into();
    // Set angular PID constants
    chassis.angular = PidBuilder { kp: 2.0, kd: 8.0, .. }.into();

    let mut robot = Robot { chassis, display: peripherals.display };

    // Calibrate the drive and spawn the odom loop, then start the competition loop
    let _odom_loop = spawn(robot.chassis.calibrate(true).await);
    robot.compete().await;
}