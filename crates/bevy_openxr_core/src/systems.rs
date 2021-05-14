use bevy_app::Events;
use bevy_ecs::ResMut;

use crate::{XRDevice, XRState, HandPoseState};

pub fn openxr_event_system(
    mut openxr: ResMut<XRDevice>,
    mut hand_pose: ResMut<HandPoseState>,
    //mut my_events: ResMut<Events<XRViewConfigurationEvent>>,
    mut state_events: ResMut<Events<XRState>>,
) {
    if let Some(inner) = openxr.inner.as_mut() {
        match inner.handle_openxr_events() {
            None => (),
            Some(changed_state) => {
                // FIXME handle XRState::Exiting
                state_events.send(changed_state);
            }
        }

        openxr.touch_update();

        if let Some(hp) = openxr.get_hand_positions() {
            *hand_pose = hp;
        }
    }
}