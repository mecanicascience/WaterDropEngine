use bevy::prelude::*;
use wde_render::{assets::{Material, MaterialBuilder, MaterialsPlugin, Mesh, Texture, TextureLoaderSettings}, pipelines::{Pipeline, PipelinesPlugin}};
use wde_wgpu::{bind_group::WBufferBindingType, render_pipeline::WShaderStages, texture::{TextureFormat, TextureUsages}};

#[derive(Asset, Clone, TypePath)]
pub struct PbrMaterial {
    pub label: String,
    pub color: (f32, f32, f32),
    pub texture: Option<Handle<Texture>>,
}
impl Default for PbrMaterial {
    fn default() -> Self {
        PbrMaterial {
            label: "pbr-material".to_string(),
            color: (1.0, 1.0, 1.0),
            texture: None,
        }
    }
}

#[repr(C)]
#[derive(Default, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct PbrMaterialUniform {
    pub color: [f32; 3],
    pub has_texture: f32,
}
impl Material for PbrMaterial {
    fn describe(&self, builder: &mut MaterialBuilder) {
        // Create uniform
        let uniform = PbrMaterialUniform {
            color: [self.color.0, self.color.1, self.color.2],
            has_texture: if self.texture.is_some() { 1.0 } else { 0.0 },
        };

        // Builder
        builder.add_buffer(
            0, WShaderStages::FRAGMENT, WBufferBindingType::Uniform,
            size_of::<PbrMaterialUniform>(), Some(bytemuck::cast_slice(&[uniform]).to_vec()));
        builder.add_texture_view(1, WShaderStages::FRAGMENT, self.texture.clone());
        builder.add_texture_sampler( 2, WShaderStages::FRAGMENT, self.texture.clone());
    }
    fn label(&self) -> &str {
        &self.label
    }
}


#[derive(Asset, Clone, TypePath)]
pub struct PbrPipeline;
impl Pipeline for PbrPipeline {}


#[derive(Bundle)]
pub struct PbrBundle {
    pub transform: Transform,
    pub mesh: Handle<Mesh>,
    pub material: Handle<PbrMaterial>,
}



pub struct MeshComponent;
impl Plugin for MeshComponent {
    fn build(&self, app: &mut App) {
        app
            .add_plugins(MaterialsPlugin::<PbrMaterial>::default())
            .add_plugins(PipelinesPlugin::<PbrPipeline, PbrMaterial>::default())
            .add_systems(Startup, init);
    }
}

fn init(mut commands: Commands, asset_server: Res<AssetServer>, mut materials: ResMut<Assets<PbrMaterial>>) {
    let box_texture = asset_server.load_with_settings("mesh/box.png", |settings: &mut TextureLoaderSettings| {
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
    let suzanne = asset_server.load("mesh/suzanne.obj");
    let cube = asset_server.load("mesh/cube.obj");

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
