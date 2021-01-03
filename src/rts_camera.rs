use bevy::input::mouse::MouseWheel;
use bevy::prelude::*;

const KEYBOARD_ZOOM_SPEED: f32 = 350.0;
const MOUSE_ZOOM_SPEED: f32 = 750.0;
const PAN_SPEED: f32 = 200.0;
const MOUSE_PAN_MARGINS: f32 = 5.0;

const LEFT_KEYS: &[KeyCode] = &[KeyCode::Left, KeyCode::A];
const RIGHT_KEYS: &[KeyCode] = &[KeyCode::Right, KeyCode::D];
const UP_KEYS: &[KeyCode] = &[KeyCode::Up, KeyCode::W];
const DOWN_KEYS: &[KeyCode] = &[KeyCode::Down, KeyCode::S];
const ZOOM_IN_KEYS: &[KeyCode] = &[KeyCode::Equals, KeyCode::NumpadAdd];
const ZOOM_OUT_KEYS: &[KeyCode] = &[KeyCode::NumpadSubtract, KeyCode::Minus];

pub struct RtsCamera {
    cursor_scroll_event_reader: EventReader<MouseWheel>,
    looking_at: Vec3,
    rotation: Quat,
    max_angle: f32,
    min_angle: f32,
    distance: f32,
    max_distance: f32,
}

impl RtsCamera {
    pub fn new_looking_at_zoomed_out(pos: Vec3, angle: f32, distance: f32) -> Self {
        RtsCamera {
            cursor_scroll_event_reader: EventReader::default(),
            looking_at: pos,
            rotation: Quat::from_rotation_x(angle),
            max_angle: 1.1637539,
            min_angle: 0.5705693,
            distance,
            max_distance: distance,
        }
    }

    pub fn camera_transform(&self) -> Transform {
        let translation = self.looking_at + self.rotation * Vec3::new(0.0, 0.0, self.distance);
        let mat4 = Mat4::from_rotation_translation(self.rotation, translation);
        Transform::from_matrix(mat4)
    }
}

pub fn rts_camera_system(
    time: Res<Time>,
    windows: Res<Windows>,
    cursor_scroll_events: Res<Events<MouseWheel>>,
    keyboard: Res<Input<KeyCode>>,
    mut query: Query<(&mut RtsCamera, &mut Transform)>,
) {
    for (mut state, mut transform) in query.iter_mut() {
        let window = windows.get_primary().unwrap();
        let cursor = match window.cursor_position() {
            Some(pos) => pos,
            None => return,
        };

        let scroll = state
            .cursor_scroll_event_reader
            .latest(&cursor_scroll_events)
            .map(|e| e.y);

        let mut translation = Vec3::new(0.0, 0.0, 0.0);
        let mut zoom_in_amount = 0f32;

        // Calculate the translation and zoom needed
        if cursor.x < MOUSE_PAN_MARGINS {
            translation.x = -1.0;
        } else if cursor.x > window.width() as f32 - MOUSE_PAN_MARGINS {
            translation.x = 1.0;
        }

        if cursor.y < MOUSE_PAN_MARGINS {
            translation.z = 1.0; // y is actually away from Earth's surface, so z is right here
        } else if cursor.y > window.height() as f32 - MOUSE_PAN_MARGINS {
            translation.z = -1.0;
        }

        if let Some(y) = scroll {
            zoom_in_amount += y * MOUSE_ZOOM_SPEED;
        }

        if LEFT_KEYS.iter().any(|c| keyboard.pressed(*c)) {
            translation.x += -1.0;
        }

        if RIGHT_KEYS.iter().any(|c| keyboard.pressed(*c)) {
            translation.x += 1.0;
        }

        if UP_KEYS.iter().any(|c| keyboard.pressed(*c)) {
            translation.z += -1.0;
        }

        if DOWN_KEYS.iter().any(|c| keyboard.pressed(*c)) {
            translation.z += 1.0;
        }

        if ZOOM_IN_KEYS.iter().any(|c| keyboard.pressed(*c)) {
            zoom_in_amount += KEYBOARD_ZOOM_SPEED;
        }

        if ZOOM_OUT_KEYS.iter().any(|c| keyboard.pressed(*c)) {
            zoom_in_amount += -KEYBOARD_ZOOM_SPEED;
        }

        // Effect of the zoom on the zoom speed
        let zoom_zoom_factor = (state.distance / state.max_distance - 1.0).tanh() + 1.0;

        // Effect of the zoom on the pan speed
        let pan_zoom_factor = f32::max(0.1, (state.distance / state.max_distance).tanh());

        let new_dist = state.distance - zoom_in_amount * time.delta_seconds() * zoom_zoom_factor;
        state.distance = f32::min(f32::max(new_dist, 70.0), 380.0);

        let translation = translation * time.delta_seconds() * PAN_SPEED * pan_zoom_factor;
        state.looking_at += translation;

        // Rotate camera angle depending on zoom
        let angle = state.distance / state.max_distance * state.max_angle;
        let angle = f32::max(f32::min(angle, state.max_angle), state.min_angle);
        state.rotation = Quat::from_rotation_x(-angle);

        *transform = state.camera_transform();
    }
}
