mod transform_component;
mod camera_component;
mod test_component;

use bevy::prelude::*;

pub use transform_component::*;
pub use camera_component::*;
pub use test_component::*;

pub struct ComponentsPlugin;
impl Plugin for ComponentsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(TestComponentPlugin);
    }
}

