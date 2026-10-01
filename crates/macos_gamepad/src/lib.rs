//! Native macOS controllers, feeding the same Bevy components that the
//! existing gameplay bindings, menus, deadzones and aim-assist already read.
use bevy::{
    input::{
        InputSystems,
        gamepad::{
            GamepadAxis, GamepadButton, GamepadConnection, GamepadConnectionEvent,
            RawGamepadAxisChangedEvent, RawGamepadButtonChangedEvent, RawGamepadEvent,
        },
    },
    prelude::*,
};
use std::collections::HashMap;

#[repr(C)]
#[derive(Clone, Copy)]
struct Snapshot {
    id: u64,
    name: [u8; 128],
    axes: [f32; 4],
    buttons: [f32; 17],
}
impl Default for Snapshot {
    fn default() -> Self {
        Self {
            id: 0,
            name: [0; 128],
            axes: [0.0; 4],
            buttons: [0.0; 17],
        }
    }
}
#[cfg(target_os = "macos")]
unsafe extern "C" {
    fn iw4l_poll_controllers(out: *mut Snapshot, capacity: u32, pump: i32) -> u32;
}

const AXES: [GamepadAxis; 4] = [
    GamepadAxis::LeftStickX,
    GamepadAxis::LeftStickY,
    GamepadAxis::RightStickX,
    GamepadAxis::RightStickY,
];
const BUTTONS: [GamepadButton; 17] = [
    GamepadButton::South,
    GamepadButton::East,
    GamepadButton::North,
    GamepadButton::West,
    GamepadButton::LeftTrigger,
    GamepadButton::RightTrigger,
    GamepadButton::LeftTrigger2,
    GamepadButton::RightTrigger2,
    GamepadButton::Select,
    GamepadButton::Start,
    GamepadButton::Mode,
    GamepadButton::LeftThumb,
    GamepadButton::RightThumb,
    GamepadButton::DPadUp,
    GamepadButton::DPadDown,
    GamepadButton::DPadLeft,
    GamepadButton::DPadRight,
];
#[derive(Default)]
struct Backend {
    pads: HashMap<u64, (Entity, Snapshot)>,
}
pub struct MacGamepadPlugin;
impl Plugin for MacGamepadPlugin {
    fn build(&self, app: &mut App) {
        app.insert_non_send(Backend::default())
            .add_systems(PreUpdate, poll.before(InputSystems));
    }
}
fn normalized(mut s: Snapshot, focused: bool) -> Snapshot {
    for v in &mut s.axes {
        *v = if focused && v.is_finite() {
            v.clamp(-1.0, 1.0)
        } else {
            0.0
        };
    }
    for v in &mut s.buttons {
        *v = if focused && v.is_finite() {
            v.clamp(0.0, 1.0)
        } else {
            0.0
        };
    }
    s
}
fn changes(entity: Entity, old: &Snapshot, new: &Snapshot) -> Vec<RawGamepadEvent> {
    let mut out = Vec::new();
    for (index, &axis) in AXES.iter().enumerate() {
        if old.axes[index] != new.axes[index] {
            out.push(RawGamepadAxisChangedEvent::new(entity, axis, new.axes[index]).into());
        }
    }
    for (index, &button) in BUTTONS.iter().enumerate() {
        if old.buttons[index] != new.buttons[index] {
            out.push(RawGamepadButtonChangedEvent::new(entity, button, new.buttons[index]).into());
        }
    }
    out
}
fn poll(
    mut commands: Commands,
    mut backend: NonSendMut<Backend>,
    windows: Query<&Window, With<bevy::window::PrimaryWindow>>,
    mut events: MessageWriter<RawGamepadEvent>,
    mut connections: MessageWriter<GamepadConnectionEvent>,
    mut buttons: MessageWriter<RawGamepadButtonChangedEvent>,
    mut axes: MessageWriter<RawGamepadAxisChangedEvent>,
) {
    let mut snapshots = [Snapshot::default(); 8];
    #[cfg(target_os = "macos")]
    // SAFETY: NonSendMut keeps this system on the main thread; repr(C) and
    // fixed capacity match native.m. The native side writes at most 8 rows.
    let count = unsafe {
        iw4l_poll_controllers(
            snapshots.as_mut_ptr(),
            snapshots.len() as u32,
            windows.is_empty() as i32,
        )
    } as usize;
    #[cfg(not(target_os = "macos"))]
    let count = 0;
    let focused = windows.iter().next().is_none_or(|w| w.focused);
    let seen: Vec<u64> = snapshots[..count.min(snapshots.len())]
        .iter()
        .map(|s| s.id)
        .collect();
    for id in backend
        .pads
        .keys()
        .copied()
        .filter(|id| !seen.contains(id))
        .collect::<Vec<_>>()
    {
        let (entity, _) = backend.pads.remove(&id).unwrap();
        let event = GamepadConnectionEvent::new(entity, GamepadConnection::Disconnected);
        events.write(event.clone().into());
        connections.write(event);
        commands.entity(entity).despawn();
    }
    for snapshot in snapshots.iter().take(count) {
        let current = normalized(*snapshot, focused);
        let entry = backend.pads.entry(snapshot.id).or_insert_with(|| {
            let end = snapshot
                .name
                .iter()
                .position(|&c| c == 0)
                .unwrap_or(snapshot.name.len());
            let name = format!(
                "{} (Apple GameController)",
                String::from_utf8_lossy(&snapshot.name[..end])
            );
            let entity = commands.spawn_empty().id();
            let event = GamepadConnectionEvent::new(
                entity,
                GamepadConnection::Connected {
                    name: name.clone(),
                    vendor_id: None,
                    product_id: None,
                },
            );
            events.write(event.clone().into());
            connections.write(event);
            diag::info!(Ui, "native controller connected: {name}");
            (entity, Snapshot::default())
        });
        for event in changes(entry.0, &entry.1, &current) {
            match &event {
                RawGamepadEvent::Button(event) => {
                    buttons.write(*event);
                }
                RawGamepadEvent::Axis(event) => {
                    axes.write(*event);
                }
                _ => {}
            }
            events.write(event);
        }
        entry.1 = current;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy::input::{InputPlugin, gamepad::Gamepad};
    #[test]
    fn native_values_reach_bevy_sticks_buttons_and_triggers() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, InputPlugin));
        let entity = app.world_mut().spawn_empty().id();
        let connected = GamepadConnectionEvent::new(
            entity,
            GamepadConnection::Connected {
                name: "test DualSense".into(),
                vendor_id: None,
                product_id: None,
            },
        );
        app.world_mut().write_message(connected.clone());
        app.world_mut()
            .write_message(RawGamepadEvent::Connection(connected));
        let mut s = Snapshot::default();
        s.axes = [1.0, -1.0, -1.0, 1.0];
        s.buttons[0] = 1.0;
        s.buttons[6] = 1.0;
        s.buttons[7] = 1.0;
        for e in changes(entity, &Snapshot::default(), &s) {
            app.world_mut().write_message(e);
        }
        app.update();
        let pad = app.world().get::<Gamepad>(entity).unwrap();
        assert_eq!(pad.left_stick(), Vec2::new(1.0, -1.0));
        assert_eq!(pad.right_stick(), Vec2::new(-1.0, 1.0));
        for button in [
            GamepadButton::South,
            GamepadButton::LeftTrigger2,
            GamepadButton::RightTrigger2,
        ] {
            assert!(pad.pressed(button));
        }
        for e in changes(entity, &s, &normalized(s, false)) {
            app.world_mut().write_message(e);
        }
        app.update();
        let pad = app.world().get::<Gamepad>(entity).unwrap();
        assert_eq!(pad.left_stick(), Vec2::ZERO);
        assert!(!pad.pressed(GamepadButton::RightTrigger2));
    }
    #[test]
    fn invalid_values_and_background_input_are_neutralized() {
        let mut s = Snapshot::default();
        s.axes = [f32::NAN, 2.0, -2.0, f32::INFINITY];
        s.buttons[7] = f32::NAN;
        let n = normalized(s, true);
        assert_eq!(n.axes, [0.0, 1.0, -1.0, 0.0]);
        assert_eq!(n.buttons[7], 0.0);
        assert_eq!(normalized(n, false).axes, [0.0; 4]);
    }
}
