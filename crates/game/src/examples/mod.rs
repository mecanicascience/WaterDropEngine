use bevy::prelude::*;

mod display_texture;

#[allow(dead_code)]
enum Examples {
    None,
    DisplayTexture,
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
            }
        }
    }
}
