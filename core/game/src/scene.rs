use std::collections::HashMap;
use std::fmt::Formatter;

use wde_ecs::EntityIndex;
use wde_ecs::World;
use wde_math::{Quatf, Rad, Rotation3, Vec2f, Vec3f, Vector3, ONE_VEC3F, QUATF_IDENTITY};
use wde_resources::ModelResource;
use wde_resources::TextureResource;
use wde_resources::ResourcesManager;
use wde_wgpu::{KeyCode, PhysicalKey};

use crate::CameraComponent;
use crate::LabelComponent;
use crate::MapComponent;
use crate::TerrainComponent;
use crate::TransformComponent;

/// Describes a scene in the game.
pub struct Scene {
    /// The world of the scene
    pub world: World,

    /// Current keys states
    keys_states: HashMap<PhysicalKey, bool>,

    /// The active camera EntityIndex
    pub active_camera: EntityIndex,
    /// The list of all cameras EntityIndex
    pub camera_list: Vec<EntityIndex>,

    // Camera controller
    last_camera_time: std::time::Instant,
    camera_rotation: Vec2f,
    camera_initial_rot: Quatf
}

impl std::fmt::Debug for Scene {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Scene")
            .field("world", &self.world)
            .field("active_camera", &self.active_camera)
            .field("camera_list", &self.camera_list)
            .finish()
    }
}

impl Scene {
    /// Create a new scene.
    /// 
    /// # Arguments
    /// 
    /// * `res_manager` - The resources manager
    #[tracing::instrument]
    pub fn new(res_manager: &mut ResourcesManager) -> Self {
        // Create world
        let mut world = World::new();
        world
            .register_component::<LabelComponent>("LabelComponent")
            .register_component::<TransformComponent>("TransformComponent")
            .register_component::<CameraComponent>("CameraComponent")
            .register_component::<MapComponent>("MapComponent")
            .register_component::<TerrainComponent>("TerrainComponent");

        // Create physics collider set
        // let mut collider_set = ColliderSet::new();

        // Create world map
        let world_map = world.create_entity().unwrap();
        world
            .add_component(world_map, LabelComponent { label : "Terrain".to_string() }).unwrap()
            .add_component(world_map, TransformComponent {
                position: Vec3f { x: 0.0, y: 0.0, z: 0.0 }, rotation: QUATF_IDENTITY, scale: ONE_VEC3F
            }).unwrap()
            .add_component(world_map, TerrainComponent {
                heightmap: res_manager.load::<TextureResource>("texture/terrain_heightmap"),
                chunks: (10, 10),
                height: 10.0
            }).unwrap()
            .add_component(world_map, MapComponent {
            }).unwrap();
        
        // // Set the world map collider
        // let heights = vec![0.0; terrain_subdivision as usize * terrain_subdivision as usize];
        // let collider = ColliderBuilder::heightfield(heights, nalgebra::vector![terrain_scale.0 / terrain_subdivision as f32, 1.0, terrain_scale.1 / terrain_subdivision as f32])
        //     .translation(nalgebra::Vector3::new(0.0, 0.0, 0.0))
        //     .build();
        
        // Create camera
        let camera = world.create_entity().unwrap();
        world
            .add_component(camera, LabelComponent { label : "Camera".to_string() }).unwrap()
            .add_component(camera, TransformComponent {
                position: Vec3f { x: 0.0, y: 2.0, z: 5.0 }, rotation: QUATF_IDENTITY, scale: ONE_VEC3F
            }).unwrap()
            .add_component(camera, CameraComponent { aspect: 1.0, fovy: 60.0, znear: 0.1, zfar: 1000.0 }).unwrap();

        // Set the camera controller : NOTE: If the active camera is changed, the controller will not work
        let last_camera_time = std::time::Instant::now();
        let camera_rotation = Vec2f { x: 0.0, y: 0.0 };
        let camera_initial_rot = world.get_component::<TransformComponent>(camera).unwrap().rotation.clone();


        // Load dummy model
        res_manager.load::<ModelResource>("models/cube");

        // Return the scene
        Scene {
            world,

            keys_states: HashMap::new(),

            camera_list: vec![camera],

            active_camera: camera,
            last_camera_time,
            camera_rotation,
            camera_initial_rot
        }
    }

    /// Update the scene.
    /// This function will update the camera controller.
    #[tracing::instrument]
    pub fn update(&mut self) {
        let move_speed = 0.01;
        let sensitivity = 0.002;

        // Update the camera controller
        {
            let mut transform = self.world.get_component::<TransformComponent>(self.active_camera).unwrap().clone();

            // Update the transform position
            let dt = ((self.last_camera_time.elapsed().as_nanos()) as f64 / 1_000_000.0) as f32;
            let forward = TransformComponent::forward(transform);
            let right = TransformComponent::right(transform);
            let up = TransformComponent::up(transform);
            if *self.keys_states.get(&PhysicalKey::Code(KeyCode::KeyW)).unwrap_or(&false) {
                transform.position += forward * move_speed * dt;
            }
            if *self.keys_states.get(&PhysicalKey::Code(KeyCode::KeyS)).unwrap_or(&false) {
                transform.position -= forward * move_speed * dt;
            }
            if *self.keys_states.get(&PhysicalKey::Code(KeyCode::KeyD)).unwrap_or(&false) {
                transform.position += right * move_speed * dt;
            }
            if *self.keys_states.get(&PhysicalKey::Code(KeyCode::KeyA)).unwrap_or(&false) {
                transform.position -= right * move_speed * dt;
            }
            if *self.keys_states.get(&PhysicalKey::Code(KeyCode::KeyE)).unwrap_or(&false) {
                transform.position += up * move_speed * dt;
            }
            if *self.keys_states.get(&PhysicalKey::Code(KeyCode::KeyQ)).unwrap_or(&false) {
                transform.position -= up * move_speed * dt;
            }

            // Get x and y rotation
            self.camera_rotation.x += if *self.keys_states.get(&PhysicalKey::Code(KeyCode::ArrowLeft)).unwrap_or(&false) {
                sensitivity * dt
            } else if *self.keys_states.get(&PhysicalKey::Code(KeyCode::ArrowRight)).unwrap_or(&false) {
                -sensitivity * dt
            } else {
                0.0
            };
            self.camera_rotation.y += if *self.keys_states.get(&PhysicalKey::Code(KeyCode::ArrowUp)).unwrap_or(&false) {
                sensitivity * dt
            } else if *self.keys_states.get(&PhysicalKey::Code(KeyCode::ArrowDown)).unwrap_or(&false) {
                -sensitivity * dt
            } else {
                0.0
            };

            // Clamp the y rotation (to avoid gimbal lock)
            let bounds_rot_y = Rad(88.0);
            self.camera_rotation.y = self.camera_rotation.y.clamp(-bounds_rot_y.0, bounds_rot_y.0);

            // Update the transform rotation
            let rot_x = Quatf::from_axis_angle(Vector3 {x:0.0, y:1.0, z:0.0}, Rad(self.camera_rotation.x));
            let rot_y = Quatf::from_axis_angle(Vector3 {x:-1.0, y:0.0, z:0.0}, Rad(-self.camera_rotation.y));
            transform.rotation = self.camera_initial_rot * rot_x * rot_y;

            // Update the transform
            self.world.set_component(self.active_camera, transform).unwrap();

            // Set the last camera time
            self.last_camera_time = std::time::Instant::now();
        }
    }

    /// Set the keys states.
    /// 
    /// # Arguments
    /// 
    /// * `keys_states` - The keys states
    pub fn set_keys_states(&mut self, keys_states: HashMap<PhysicalKey, bool>) {
        self.keys_states = keys_states;
    }
}
