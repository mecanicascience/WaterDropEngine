use bevy::prelude::*;
use wde_render::assets::{Texture, TextureFormat, TextureLoaderSettings};

pub struct TestComponentPlugin;
impl Plugin for TestComponentPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(Startup, init_test)
            .add_systems(Update, test_kill_entity);
        app.init_resource::<Counter>();
    }
}

#[derive(Component, Default)]
pub struct TestComponent {
    pub heightmap: Handle<Texture>,
}

fn init_test(mut commands: Commands, server: Res<AssetServer>) {
    let heightmap: Handle<Texture> = server.load_with_settings("test/heightmap.png", |settings: &mut TextureLoaderSettings| {
        settings.label = "Heightmap".to_string();
        settings.format = TextureFormat::R8Unorm;
        settings.force_depth = Some(1);
    });

    // Create the terrain entity
    commands.spawn(TestComponent { heightmap });
}

#[derive(Resource, Default)]
pub struct Counter(u32);

fn test_kill_entity(mut commands: Commands, test_elements: Query<(Entity, &TestComponent)>, mut counter: ResMut<Counter>) {
    if counter.0 == 300 {
        for (entity, _) in test_elements.iter() {
            commands.entity(entity).despawn();
        }
    }
    counter.0 += 1;
}
