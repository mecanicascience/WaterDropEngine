use bevy::prelude::*;

use crate::{renderer::{extract_macros::*, Extract, Render, RenderApp, RenderSet}, scene::components::{CameraComponent, CameraViewComponent, TransformComponent}};

pub struct CameraFeature;
impl Plugin for CameraFeature {
    fn build(&self, app: &mut App) {
        let render_app = app.get_sub_app_mut(RenderApp).unwrap();

        // Register systems
        render_app.add_systems(Extract, extract_camera);

        // TEMP TEMP TEMP
        render_app.add_systems(Render, log_camera_component.in_set(RenderSet::Render));
    }
}



//////////// TEMP TEMP TEMP TEMP TEMP TEMP TEMP TEMP TEMP TEMP
fn log_camera_component(_query: Query<(&TransformComponent, &CameraViewComponent)>) {
    // println!("Logging camera component");
    // for (transform, view) in query.iter() {
    //     println!("Transform: {:?}", (transform.position, transform.rotation, transform.scale));
    //     println!("View: {:?}", (view.fov, view.near, view.far));
    // }
}

fn extract_camera(mut commands: Commands, query: ExtractWorld<Query<(Entity, &TransformComponent, &CameraViewComponent)>>) {
    // println!("Extracting camera");

    // Add command to replace camera on next frame
    for (entity, transform, view) in query.iter() {
        // Insert extracted camera component at the index of the main world entity
        commands.get_or_spawn(entity).insert(CameraComponent {
            transform: *transform,
            view: *view,
        });

        // println!("Extracting camera");
        // println!("Transform: {:?}", (transform.position, transform.rotation, transform.scale));
        // println!("View: {:?}", (view.fov, view.near, view.far));
    }
}