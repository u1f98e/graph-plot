use bevy::{prelude::*, input::mouse::MouseWheel, window::PrimaryWindow};

pub fn key_input(key: Res<Input<KeyCode>>) {
	if key.just_pressed(KeyCode::Escape) {
		std::process::exit(0);
	}
}

pub fn mouse_button_input(buttons: Res<Input<MouseButton>>) {
	if buttons.just_pressed(MouseButton::Left) {
	}
}

pub fn mouse_scroll_input(mut scroll_evr: EventReader<MouseWheel>) {

}

pub fn mouse_position(windows_q: Query<&Window, With<PrimaryWindow>>) {

}