use bevy::prelude::*;

mod display_texture;
mod pbr_batches;
mod custom_forward_render;

#[allow(dead_code)]
enum Examples {
    /// No example selected
    None,
    /// Display a texture onto the screen view
    DisplayTexture,
    /// Create a scene with pbr batches of entities with different pbr materials and meshes
    PbrBatches,
    /// Implementation of a custom forward render pass, pipeline and material
    CustomForwardRender
}

pub struct ExamplesPugin;
impl Plugin for ExamplesPugin {
    fn build(&self, app: &mut App) {
        let selected_example = Examples::None;

        // Load the selected example
        match selected_example {
            Examples::None => {}
            Examples::DisplayTexture => {
                app.add_plugins(display_texture::DisplayTextureComponentPlugin)
                    .add_plugins(display_texture::DisplayTextureFeature);
            },
            Examples::PbrBatches => {
                app.add_plugins(pbr_batches::PbrBatchesPlugin);
            },
            Examples::CustomForwardRender => {
                app.add_plugins(custom_forward_render::PbrFeaturesPlugin);
            }
        }
    }
}
