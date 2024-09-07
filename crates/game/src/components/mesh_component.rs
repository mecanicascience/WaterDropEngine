use bevy::prelude::*;
use wde_render::assets::{MaterialType, Mesh};

#[derive(Reflect, Clone)]
pub struct PbrMaterial {
    pub color: (f32, f32, f32),
}
impl MaterialType for PbrMaterial {}

pub struct MeshComponent;
impl Plugin for MeshComponent {
    fn build(&self, app: &mut App) {
        app
            // .init_asset::<Material<PbrMaterial>>()
            // .add_plugins(RenderAssetsPlugin::<GpuMaterial<PbrMaterial>>::default())
            .add_systems(Startup, init);
    }
}


#[derive(Component)]
pub struct DrawItem {
    pub mesh: Handle<Mesh>,
    // pub material: Handle<Material<PbrMaterial>>,
}

#[derive(Bundle)]
pub struct PbrBundle {
    pub transform: Transform,
    pub mesh: Handle<Mesh>,
}

fn init(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Creates a mesh
    commands.spawn(PbrBundle {
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        mesh: asset_server.load("mesh/suzanne.obj"),
        // material: asset_server.add(Material::<PbrMaterial> {
        //     label: "pbr_material".to_string(),
        //     ..Default::default()
        // })
    });
}
