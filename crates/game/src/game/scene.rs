use bevy::prelude::*;
use wde_render::{assets::TextureLoaderSettings, components::{Camera, CameraController, CameraView}, renderer::pbr::{PbrBundle, PbrMaterial}};
use wde_wgpu::texture::{TextureFormat, TextureUsages};

pub struct GamePlugin;
impl Plugin for GamePlugin {
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
    let box_texture = asset_server.load_with_settings("models/box.png", |settings: &mut TextureLoaderSettings| {
        settings.label = "pbr-box".to_string();
        settings.format = TextureFormat::Rgba8Unorm;
        settings.usages = TextureUsages::TEXTURE_BINDING;
    });
    let red_box = materials.add(PbrMaterial {
        label: "pbr-material-red-box".to_string(),
        color: (1.0, 0.0, 0.0),
        texture: Some(box_texture),
    });
    let blue = materials.add(PbrMaterial {
        label: "pbr-material-blue".to_string(),
        color: (0.0, 0.0, 1.0),
        texture: None,
    });
    let suzanne = asset_server.load("models/suzanne.obj");
    let cube = asset_server.load("models/cube.obj");

    // Spawn the entities
    commands.spawn(PbrBundle {
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        mesh: cube.clone(),
        material: red_box.clone()
    });
    commands.spawn(PbrBundle {
        transform: Transform::from_xyz(5.0, 0.0, 0.0),
        mesh: cube.clone(),
        material: red_box.clone()
    });
    commands.spawn(PbrBundle {
        transform: Transform::from_xyz(10.0, 0.0, 0.0),
        mesh: cube.clone(),
        material: blue.clone()
    });
    commands.spawn(PbrBundle {
        transform: Transform::from_xyz(15.0, 0.0, 0.0),
        mesh: suzanne.clone(),
        material: blue.clone()
    });
    commands.spawn(PbrBundle {
        transform: Transform::from_xyz(20.0,0.0, 0.0),
        mesh: cube.clone(),
        material: blue.clone()
    });
    commands.spawn(PbrBundle {
        transform: Transform::from_xyz(25.0, 0.0, 0.0),
        mesh: cube.clone(),
        material: red_box.clone()
    });
    for i in 1..100 {
        for j in 1..100 {
            commands.spawn(PbrBundle {
                transform: Transform::from_xyz(i as f32 * 5.0, 0.0, j as f32 * 5.0 + 5.0),
                mesh: suzanne.clone(),
                material: red_box.clone()
            });
        }
    }
}
