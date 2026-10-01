//! Read-only hardware-to-Bevy check. No renderer, match or synthetic input.
use bevy::{
    app::ScheduleRunnerPlugin,
    input::{InputPlugin, gamepad::Gamepad},
    prelude::*,
};
#[derive(Resource)]
struct Probe {
    start: std::time::Instant,
    last: String,
    changes: u32,
}
fn main() {
    App::new()
        .add_plugins((
            MinimalPlugins.set(ScheduleRunnerPlugin::run_loop(
                std::time::Duration::from_secs_f64(1.0 / 60.0),
            )),
            InputPlugin,
            macos_gamepad::MacGamepadPlugin,
        ))
        .insert_resource(Probe {
            start: std::time::Instant::now(),
            last: String::new(),
            changes: 0,
        })
        .add_systems(Update, read)
        .run();
}
fn read(pads: Query<&Gamepad>, mut probe: ResMut<Probe>, mut exit: MessageWriter<AppExit>) {
    for pad in &pads {
        let state = format!(
            "L={:.2},{:.2} R={:.2},{:.2} buttons={:?}",
            pad.left_stick().x,
            pad.left_stick().y,
            pad.right_stick().x,
            pad.right_stick().y,
            pad.get_pressed().collect::<Vec<_>>()
        );
        if state != probe.last {
            println!("{state}");
            probe.last = state;
            probe.changes += 1;
        }
    }
    if probe.start.elapsed().as_secs() >= 30 {
        println!(
            "connected={} state_changes={}",
            pads.iter().count(),
            probe.changes
        );
        exit.write(AppExit::Success);
    }
}
