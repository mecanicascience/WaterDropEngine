use bevy::prelude::*;
use wde_render::{assets::TextureLoaderSettings, components::{Camera, CameraController, CameraView}, renderer::pbr::{PbrBundle, PbrMaterial}};
use wde_wgpu::texture::{WTextureFormat, WTextureUsages};

pub struct PbrBatchesPlugin;
impl Plugin for PbrBatchesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, init);
    }
}

fn init(mut commands: Commands, asset_server: Res<AssetServer>, mut materials: ResMut<Assets<PbrMaterial>>) {
    // Creates a camera
    commands.spawn(
        (Camera {
            transform: Transform::from_xyz(2.0, 2.0, 2.0).looking_at(Vec3::ZERO, Vec3::Y),
            view: CameraView::default()
        },
        CameraController::default()
    ));

    // Load the assets
    let box_texture = asset_server.load_with_settings("examples/pbr_batches/box.png", |settings: &mut TextureLoaderSettings| {
        settings.label = "pbr-box".to_string();
        settings.format = WTextureFormat::Rgba8Unorm;
        settings.usages = WTextureUsages::TEXTURE_BINDING;
    });
    let red_box = materials.add(PbrMaterial {
        label: "pbr-material-red-box".to_string(),
        albedo: (1.0, 0.0, 0.0),
        texture: Some(box_texture),
        ..Default::default()
    });
    let blue = materials.add(PbrMaterial {
        label: "pbr-material-blue".to_string(),
        albedo: (0.0, 0.0, 1.0),
        texture: None,
        ..Default::default()
    });
    let suzanne = asset_server.load("examples/pbr_batches/suzanne.obj");
    let cube = asset_server.load("examples/pbr_batches/cube.obj");

    // Spawn the entities
    commands.spawn(PbrBundle {
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        mesh: cube.clone(),
        material: blue.clone()
    });
    commands.spawn(PbrBundle {
        transform: Transform::from_xyz(5.0, 0.0, 0.0),
        mesh: cube.clone(),
        material: blue.clone()
    });
    commands.spawn(PbrBundle {
        transform: Transform::from_xyz(10.0, 0.0, 0.0),
        mesh: cube.clone(),
        material: red_box.clone()
    });
    commands.spawn(PbrBundle {
        transform: Transform::from_xyz(15.0, 0.0, 0.0),
        mesh: suzanne.clone(),
        material: red_box.clone()
    });
    for i in 1..100 {
        for j in 1..100 {
            commands.spawn(PbrBundle {
                transform: Transform::from_xyz(i as f32 * 5.0, 0.0, j as f32 * 5.0 + 5.0),
                mesh: suzanne.clone(),
                material: blue.clone()
            });
        }
    }
}
