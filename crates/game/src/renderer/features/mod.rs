mod camera_feature;
mod test_feature;

use bevy::app::{App, Plugin};
pub use camera_feature::*;
pub use test_feature::*;

pub struct RenderFeaturesPlugin;
impl Plugin for RenderFeaturesPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(CameraFeature)
            .add_plugins(TestFeature);
    }
}
