use bevy::prelude::*;
use wde_render::{assets::TextureLoaderSettings, components::{Camera, CameraController, CameraView}, renderer::pbr::{PbrBundle, PbrMaterial}};
use wde_wgpu::texture::{WTextureFormat, TextureUsages};

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
        settings.format = WTextureFormat::Rgba8Unorm;
        settings.usages = TextureUsages::TEXTURE_BINDING;
    });
    let red_box = materials.add(PbrMaterial {
        label: "pbr-material-red-box".to_string(),
        albedo: (1.0, 0.0, 0.0),
        texture: Some(box_texture),
        metallic: 1.0,
        roughness: 0.2,
        reflectance: 0.0,
    });
    let blue = materials.add(PbrMaterial {
        label: "pbr-material-blue".to_string(),
        albedo: (0.0, 0.0, 1.0),
        texture: None,
        metallic: 0.0,
        roughness: 0.5,
        reflectance: 0.4,
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
            let angle = i as f32 * 0.05;
            let axis = if i % 2 == 0 { Vec3::Y } else if i % 3 == 0 { Vec3::Z } else { Vec3::X };
            commands.spawn(PbrBundle {
                transform: Transform::from_xyz(i as f32 * 5.0, 0.0, j as f32 * 5.0 + 5.0)
                    .with_rotation(Quat::from_axis_angle(axis, angle)),
                mesh: suzanne.clone(),
                material: red_box.clone()
            });
        }
    }
}
