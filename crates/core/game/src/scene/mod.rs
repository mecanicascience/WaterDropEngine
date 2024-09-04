use bevy::prelude::*;
use components::ComponentsPlugin;

pub mod components;
pub mod resources;

pub struct ScenePlugin;
impl Plugin for ScenePlugin {
    fn build(&self, app: &mut App) {
        // Setup the resources
        app.add_plugins(resources::SceneResourcesPlugin);

        // Add the components
        app.add_plugins(ComponentsPlugin);
    }
}
