use bevy::input::mouse::MouseWheel;
use bevy::{prelude::*, window::CursorMoved};

const MOUSE_SCROLL_SPEED: f32 = 250.0;
const MOUSE_PAN_SPEED: f32 = 200.0;
const MOUSE_PAN_MARGINS: f32 = 5.0;

#[derive(Default)]
pub struct State {
    cursor_moved_event_reader: EventReader<CursorMoved>,
    cursor_scroll_event_reader: EventReader<MouseWheel>,
    pos: Vec2,
}

pub fn rts_camera_system(
    time: Res<Time>,
    windows: Res<Windows>,
    cursor_moved_events: Res<Events<CursorMoved>>,
    cursor_scroll_events: Res<Events<MouseWheel>>,
    mut query: Query<(&mut State, &mut Transform)>,
) {
    for (mut state, mut transform) in &mut query.iter() {
        if let Some(event) = state.cursor_moved_event_reader.latest(&cursor_moved_events) {
            state.pos.set_x(event.position.x());
            state.pos.set_y(event.position.y());
        }

        let scroll = state
            .cursor_scroll_event_reader
            .latest(&cursor_scroll_events)
            .map(|e| e.y);
        let window = windows.get_primary().unwrap();

        let mut translation = Vec3::new(0.0, 0.0, 0.0);

        // Calculate the translation needed
        if state.pos.x() < MOUSE_PAN_MARGINS {
            translation.set_x(-1.0);
        } else if state.pos.x() > window.width as f32 - MOUSE_PAN_MARGINS {
            translation.set_x(1.0);
        }

        if state.pos.y() < MOUSE_PAN_MARGINS {
            translation.set_z(1.0); // y is actually away from Earth's surface, so z is right here
        } else if state.pos.y() > window.height as f32 - MOUSE_PAN_MARGINS {
            translation.set_z(-1.0);
        }

        if let Some(y) = scroll {
            let zoom = transform.rotation().mul_vec3(Vec3::new(0.0, -1.0, -2.0));
            transform.translate(zoom * y * MOUSE_SCROLL_SPEED * time.delta_seconds);
        }

        let zoom_factor = f32::max(1.0, transform.scale().length() - 0.05);
        let translation = translation * time.delta_seconds * MOUSE_PAN_SPEED / zoom_factor;

        // Apply movement to camera
        transform.translate(translation);
    }
}
