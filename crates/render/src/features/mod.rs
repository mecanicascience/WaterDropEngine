mod camera_feature;

use bevy::app::{App, Plugin};
pub use camera_feature::*;

pub struct RenderFeaturesPlugin;
impl Plugin for RenderFeaturesPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(CameraFeature);
    }
}
