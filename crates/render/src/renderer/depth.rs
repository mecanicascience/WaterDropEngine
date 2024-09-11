use bevy::{prelude::*, window::WindowResized};
use wde_wgpu::texture::WTexture;

use crate::{assets::{Texture, TextureUsages}, core::extract_macros::ExtractWorld};

#[derive(Resource)]
pub struct DepthTexture {
    pub texture: Handle<Texture>
}
impl DepthTexture {
    pub fn init_depth(mut commands: Commands, server: Res<AssetServer>, window: Query<&Window>) {
        let resolution = &window.single().resolution;
        let texture = server.add(Texture {
            label: "depth".to_string(),
            size: (resolution.width() as usize, resolution.height() as usize, 1),
            format: WTexture::DEPTH_FORMAT,
            usages: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            ..Default::default()
        });
        commands.insert_resource(DepthTexture { texture });
    }

    pub fn resize_depth(mut commands: Commands, mut window_resized_events: EventReader<WindowResized>, server: Res<AssetServer>) {
        for event in window_resized_events.read() {
            // Recreate the depth texture
            let texture = server.add(Texture {
                label: "depth".to_string(),
                size: (event.width as usize, event.height as usize, 1),
                format: WTexture::DEPTH_FORMAT,
                usages: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
                ..Default::default()
            });
            commands.insert_resource(DepthTexture { texture });
        }
    }

    pub fn extract_depth_texture(mut commands: Commands, depth_texture : ExtractWorld<Res<DepthTexture>>) {
        commands.insert_resource(DepthTexture { texture: depth_texture.texture.clone() });
    }
}
