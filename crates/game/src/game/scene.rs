use bevy::prelude::*;
use wde_render::{assets::TextureLoaderSettings, components::{Camera, CameraController, CameraView, DirectionalLight, PointLight, SpotLight}, renderer::pbr::{PbrBundle, PbrMaterial}};
use wde_wgpu::texture::{WTextureFormat, WTextureUsages};

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
    let container_albedo = asset_server.load_with_settings("models/container_albedo.png", |settings: &mut TextureLoaderSettings| {
        settings.label = "container-albedo".to_string();
        settings.format = WTextureFormat::Rgba8UnormSrgb;
        settings.usages = WTextureUsages::TEXTURE_BINDING;
    });
    let container_specular = asset_server.load_with_settings("models/container_specular.png", |settings: &mut TextureLoaderSettings| {
        settings.label = "container-specular".to_string();
        settings.format = WTextureFormat::R8Unorm;
        settings.usages = WTextureUsages::TEXTURE_BINDING;
    });
    let red_box = materials.add(PbrMaterial {
        label: "container".to_string(),
        albedo_t:   Some(container_albedo),
        specular_t: Some(container_specular),
        ..Default::default()
    });
    let blue = materials.add(PbrMaterial {
        label: "blue".to_string(),
        albedo: (0.0, 0.0, 1.0, 1.0),
        specular: 0.5,
        ..Default::default()
    });
    let suzanne = asset_server.load("models/suzanne.obj");
    let cube = asset_server.load("models/container.obj");

    // Spawn the entities
    commands.spawn(PbrBundle {
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        mesh: cube.clone(),
        material: red_box.clone()
    });
    commands.spawn(PbrBundle {
        transform: Transform::from_xyz(5.0, 0.0, 0.0).with_scale(Vec3::splat(3.0)),
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
                mesh: cube.clone(),
                material: red_box.clone()
            });
        }
    }

    // Spawn the lights
    commands.spawn(PointLight {
        position: Vec3::new(440.0, 30.0, 350.0),
        ..Default::default()
    }.with_range(200.0).unwrap());
    commands.spawn(SpotLight {
        position: Vec3::new(120.0, 20.0, 56.0),
        direction: Vec3::new(-0.8, -0.5, 0.0).normalize(),
        ..Default::default()
    });
    commands.spawn(SpotLight {
        position: Vec3::new(220.0, 20.0, 156.0),
        direction: Vec3::new(0.8, -0.5, -0.1).normalize(),
        ..Default::default()
    });
    commands.spawn(DirectionalLight {
        direction: Vec3::new(-0.2, 0.8, -0.3).normalize(),
        ..Default::default()
    });
}
