use std::collections::HashMap;
use std::fmt::Formatter;

use wde_ecs::{CameraComponent, EntityIndex, LabelComponent, RenderComponent, RenderComponentInstanced, RenderComponentSSBODynamic, RenderComponentSSBOStatic, TransformComponent, World};
use wde_logger::throw;
use wde_math::{Quatf, Rad, Rotation3, Vec2f, Vec3f, Vector3, ONE_VEC3F, QUATF_IDENTITY};
use wde_resources::{MaterialResource, ModelResource};
use wde_resources::ResourcesManager;
use wde_wgpu::{KeyCode, PhysicalKey};

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
    camera_initial_rot: Quatf,
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
            .register_component::<LabelComponent>()
            .register_component::<TransformComponent>()
            .register_component::<CameraComponent>()
            .register_component::<RenderComponent>()
            .register_component::<RenderComponentInstanced>()
            .register_component::<RenderComponentSSBODynamic>()
            .register_component::<RenderComponentSSBOStatic>();
        let mut render_index = 0;


        // // Create big model
        // let big_model = match world.create_entity() {
        //     Some(e) => e,
        //     None => throw!("Failed to create entity. No more entity slots available."),
        // };

        // // Set model to big model
        // world
        //     .add_component(big_model, LabelComponent { label : "Big model".to_string() }).unwrap()
        //     .add_component(big_model, TransformComponent {
        //         position: Vec3f { x: 0.0, y: 0.0, z: 0.0 }, rotation: QUATF_IDENTITY, scale: ONE_VEC3F * 0.3
        //     }).unwrap()
        //     .add_component(big_model, RenderComponentDynamic {
        //         ids: 0..1,
        //         model: res_manager.load::<ModelResource>("models/lost_empire"),
        //         material: res_manager.load::<MaterialResource>("materials/unicolor")
        //     }).unwrap();


        // Create cube
        {
            let entity = match world.create_entity() {
                Some(e) => e,
                None => throw!("Failed to create entity. No more entity slots available."),
            };
            world
                .add_component(entity, LabelComponent { label : "Cube".to_string() }).unwrap()
                .add_component(entity, TransformComponent {
                    position: Vec3f { x: -0.5, y: 0.0, z: 0.0 }, rotation: QUATF_IDENTITY, scale: ONE_VEC3F * 0.3
                }).unwrap()
                .add_component(entity, RenderComponentSSBODynamic { id: render_index }).unwrap()
                .add_component(entity, RenderComponent {
                    id: render_index,
                    model: res_manager.load::<ModelResource>("models/cube"),
                    material: res_manager.load::<MaterialResource>("materials/unicolor")
                }).unwrap();
            render_index += 1;
        }

        // Create cube
        {
            let entity = match world.create_entity() {
                Some(e) => e,
                None => throw!("Failed to create entity. No more entity slots available."),
            };
            world
                .add_component(entity, LabelComponent { label : "Cube 2".to_string() }).unwrap()
                .add_component(entity, TransformComponent {
                    position: Vec3f { x: -2.5, y: 0.0, z: 0.0 }, rotation: QUATF_IDENTITY, scale: ONE_VEC3F * 0.4
                }).unwrap()
                .add_component(entity, RenderComponentSSBODynamic { id: render_index }).unwrap()
                .add_component(entity, RenderComponent {
                    id: render_index,
                    model: res_manager.load::<ModelResource>("models/cube"),
                    material: res_manager.load::<MaterialResource>("materials/unicolor")
                }).unwrap();
            render_index += 1;
        }

        
        // Create nxn monkeys
        let n = 10;
        let mut monkey_indices = Vec::new();
        for i in 0..n {
            for j in 0..n {
                let entity = match world.create_entity() {
                    Some(e) => e,
                    None => throw!("Failed to create entity. No more entity slots available."),
                };

                // Create monkey
                world
                    .add_component(entity, LabelComponent { label : format!("Monkey {}", render_index) }).unwrap()
                    .add_component(entity, TransformComponent {
                        position: Vec3f { x: i as f32 * 1.0 - (n as f32)/2.0, y: 0.0, z: j as f32 * 1.0 - (n as f32)/2.0 }, rotation: QUATF_IDENTITY, scale: ONE_VEC3F * 0.3
                    }).unwrap()
                    .add_component(entity, RenderComponentSSBOStatic { id: render_index }).unwrap();
                monkey_indices.push(render_index);
                render_index += 1;
            }
        }
        // Add parent monkey
        {
            let entity = match world.create_entity() {
                Some(e) => e,
                None => throw!("Failed to create entity. No more entity slots available."),
            };
            world
                .add_component(entity, LabelComponent { label : "Parent monkey".to_string() }).unwrap()
                .add_component(entity, TransformComponent {
                    position: Vec3f { x: 0.0, y: 0.0, z: 0.0 }, rotation: QUATF_IDENTITY, scale: ONE_VEC3F * 0.3
                }).unwrap()
                .add_component(entity, RenderComponentInstanced {
                    ids: monkey_indices.clone().into_iter().min().unwrap()..monkey_indices.clone().into_iter().max().unwrap() + 1,
                    model: res_manager.load::<ModelResource>("models/monkey"),
                    material: res_manager.load::<MaterialResource>("materials/unicolor")
                }).unwrap();
            render_index += 1;
        }


        // Create cube
        {
            let entity = match world.create_entity() {
                Some(e) => e,
                None => throw!("Failed to create entity. No more entity slots available."),
            };
            world
                .add_component(entity, LabelComponent { label : "Cube".to_string() }).unwrap()
                .add_component(entity, TransformComponent {
                    position: Vec3f { x: 0.0, y: 0.0, z: 0.5 }, rotation: QUATF_IDENTITY, scale: ONE_VEC3F * 0.2
                }).unwrap()
                .add_component(entity, RenderComponentSSBODynamic { id: render_index }).unwrap()
                .add_component(entity, RenderComponent {
                    id: render_index,
                    model: res_manager.load::<ModelResource>("models/cube"),
                    material: res_manager.load::<MaterialResource>("materials/unicolor")
                }).unwrap();
            // render_index += 1;
        }



        // Create camera
        let camera = match world.create_entity() {
            Some(e) => e,
            None => throw!("Failed to create entity. No more entity slots available."),
        };
        world
            .add_component(camera, LabelComponent { label : "Camera".to_string() }).unwrap()
            .add_component(camera, TransformComponent {
                position: Vec3f { x: 0.0, y: 0.0, z: 0.0 }, rotation: QUATF_IDENTITY, scale: ONE_VEC3F
            }).unwrap()
            .add_component(camera, CameraComponent { aspect: 1.0, fovy: 60.0, znear: 0.1, zfar: 100.0 }).unwrap();

        // Set the camera controller : NOTE: If the active camera is changed, the controller will not work
        let last_camera_time = std::time::Instant::now();
        let camera_rotation = Vec2f { x: 0.0, y: 0.0 };
        let camera_initial_rot = world.get_component::<TransformComponent>(camera).unwrap().rotation.clone();

        // Return the scene
        Scene {
            world,

            keys_states: HashMap::new(),

            camera_list: vec![camera],

            active_camera: camera,
            last_camera_time,
            camera_rotation,
            camera_initial_rot,
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
