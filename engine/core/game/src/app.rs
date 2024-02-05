use std::{collections::HashMap, sync::{Arc, RwLock}};

use tokio::sync::mpsc;
use tracing::{span, Level};
use wde_ecs::{CameraComponent, CameraUniform, LabelComponent, RenderComponent, RenderComponentInstanced, RenderComponentSSBODynamic, RenderComponentSSBOStatic, TransformComponent, World};
use wde_logger::{info, throw, trace, debug};
use wde_editor_interactions::EditorHandler;
use wde_math::{Quatf, Rad, Rotation3, Vec2f, Vec3f, Vector3, ONE_VEC3F, QUATF_IDENTITY};
use wde_resources::{MaterialResource, ModelResource, ResourcesManager};
use wde_wgpu::{Buffer, ElementState, Event, KeyCode, LoopEvent, PhysicalKey, RenderEvent, RenderInstance, Window, WindowEvent};

use crate::Renderer;

pub struct App {}

impl App {
    pub async fn new() -> Self {
        // Configure environment if on debug mode
        if cfg!(debug_assertions) {
            // Set the RUST_BACKTRACE environment variable to 1
            std::env::set_var("RUST_BACKTRACE", "0");
        }
        debug!("======== Starting engine ========");


        // ======== WINDOW CREATION ========
        let _window_creation_span = span!(Level::INFO, "window_creation").entered();
        // Create window
        let mut window = Window::new(800, 600, "WaterDropEngine");

        // Create channel to send the window to the runtime
        let (window_t, mut window_r) = mpsc::unbounded_channel();

        // Run window
        trace!("Starting window.");
        let (event_t, mut event_r) = mpsc::unbounded_channel();
        let (event_relay_t, mut event_relay_r) = mpsc::unbounded_channel();
        let (input_t, mut input_r) = mpsc::unbounded_channel();
        let (resize_t, mut resize_r) = mpsc::unbounded_channel();
        let window_join = std::thread::spawn(move || {
            // Create event loop
            let event_loop = window.create();
            let window_index = window.run(&event_loop);

            // Send window to another channel
            window_t.send(window).unwrap();

            // Run event loop
            event_loop.run(move |event, elwt| {
                // Handle event
                match event {
                    // Close window when the close button is pressed
                    Event::WindowEvent {
                        event: WindowEvent::CloseRequested,
                        window_id,
                    } if window_id == window_index => {
                        info!("Closing window.");
                        event_t.send(LoopEvent::Close).unwrap_or_else(|e| {
                            throw!("Failed to send close event : {}.", e);
                        });
                        elwt.exit();
                    },

                    // Resize window when requested
                    Event::WindowEvent {
                        event: WindowEvent::Resized(size),
                        window_id,
                    } if window_id == window_index => {
                        resize_t.send((size.width, size.height)).unwrap_or_else(|e| {
                            throw!("Failed to send resize event : {}.", e);
                        });
                    },

                    // Check for input events
                    Event::WindowEvent {
                        event: WindowEvent::KeyboardInput {
                            event,
                            ..
                        },
                        ..
                    } => {
                        input_t.send(event).unwrap_or_else(|e| {
                            throw!("Failed to send input event : {}.", e);
                        });
                    },

                    // Ask for redraw when all events are processed
                    Event::AboutToWait => {
                        if event_relay_r.try_recv().is_ok() {
                            let _event_loop_wait_span = span!(Level::INFO, "event_loop_wait").entered();
                            event_t.send(LoopEvent::Redraw).unwrap_or_else(|e| {
                                throw!("Failed to send redraw event : {}.", e);
                            });
                        }
                    },

                    // Redraw window when requested
                    Event::WindowEvent {
                        event: WindowEvent::RedrawRequested,
                        ..
                    } => {
                        if event_relay_r.try_recv().is_ok() {
                            let _event_loop_wait_span = span!(Level::INFO, "event_loop_wait").entered();
                            event_t.send(LoopEvent::Redraw).unwrap_or_else(|e| {
                                throw!("Failed to send redraw event : {}.", e);
                            });
                        }
                    },

                    // Ignore other events
                    _ => ()
                }
            }).unwrap_or_else(|e| {
                throw!("Failed to run event loop : {:?}.", e);
            });
        });
        drop(_window_creation_span);


        // ======== ENGINE INITIALIZATION ========
        let _engine_initialization_span = span!(Level::INFO, "engine_initialization").entered();

        // Create editor handler
        let mut editor_handler: Option<EditorHandler> = if cfg!(debug_assertions) != true { None } else {
            match EditorHandler::new() {
                Ok(h) => if h.started() { Some(h) } else { None }
                Err(_) => None
            }
        };

        // Create list of input keys
        let mut keys_states: HashMap<PhysicalKey, bool> = HashMap::new();


        // Create resource manager
        let mut res_manager = ResourcesManager::new();
        
        // Wait for window
        let mut window = window_r.recv().await.unwrap();

        // Create render instance
        let mut render_instance = RenderInstance::new("WaterDropEngine", Some(&window)).await;

        // Handle editor messages and push new frame
        if editor_handler.is_some() {
            let editor = editor_handler.as_mut().unwrap();
            match editor.process() {
                Ok(_) => {
                    // Set last frame
                    let r = rand::random::<u32>();
                    let _ = editor.set_current_frame(format!("Hello {} world.", r).as_bytes());
                },
                Err(_) => {}
            }
        }
        drop(_engine_initialization_span);


        // ======== WORLD CONTENT ========
        let _world_content_span = span!(Level::INFO, "world_content").entered();

        // Create world
        let mut world = World::new();
        world
            .register_component::<LabelComponent>()
            .register_component::<TransformComponent>()
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
        let cube = match world.create_entity() {
            Some(e) => e,
            None => throw!("Failed to create entity. No more entity slots available."),
        };
        world
            .add_component(cube, LabelComponent { label : "Cube".to_string() }).unwrap()
            .add_component(cube, TransformComponent {
                position: Vec3f { x: -0.5, y: 0.0, z: 0.0 }, rotation: QUATF_IDENTITY, scale: ONE_VEC3F * 0.3
            }).unwrap()
            .add_component(cube, RenderComponentSSBODynamic { id: render_index }).unwrap()
            .add_component(cube, RenderComponent {
                id: render_index,
                model: res_manager.load::<ModelResource>("models/cube"),
                material: res_manager.load::<MaterialResource>("materials/unicolor")
            }).unwrap();
        render_index += 1;

        // Create cube 2
        let cube2 = match world.create_entity() {
            Some(e) => e,
            None => throw!("Failed to create entity. No more entity slots available."),
        };
        world
            .add_component(cube2, LabelComponent { label : "Cube 2".to_string() }).unwrap()
            .add_component(cube2, TransformComponent {
                position: Vec3f { x: -2.5, y: 0.0, z: 0.0 }, rotation: QUATF_IDENTITY, scale: ONE_VEC3F * 0.4
            }).unwrap()
            .add_component(cube2, RenderComponentSSBODynamic { id: render_index }).unwrap()
            .add_component(cube2, RenderComponent {
                id: render_index,
                model: res_manager.load::<ModelResource>("models/cube"),
                material: res_manager.load::<MaterialResource>("materials/unicolor")
            }).unwrap();
        render_index += 1;

        
        // Create nxn monkeys
        let n = 20;
        let mut indices = Vec::new();
        let mut monkeys = Vec::new();
        for i in 0..n {
            for j in 0..n {
                let monkey = match world.create_entity() {
                    Some(e) => e,
                    None => throw!("Failed to create entity. No more entity slots available."),
                };

                // Create monkey
                world
                    .add_component(monkey, LabelComponent { label : format!("Monkey {}", render_index) }).unwrap()
                    .add_component(monkey, TransformComponent {
                        position: Vec3f { x: i as f32 * 1.0 - (n as f32)/2.0, y: 0.0, z: j as f32 * 1.0 - (n as f32)/2.0 }, rotation: QUATF_IDENTITY, scale: ONE_VEC3F * 0.3
                    }).unwrap()
                    .add_component(monkey, RenderComponentSSBOStatic { id: render_index }).unwrap();
                indices.push(render_index);
                monkeys.push(monkey);
                render_index += 1;
            }
        }
        // Add parent monkey
        let parent_monkey = match world.create_entity() {
            Some(e) => e,
            None => throw!("Failed to create entity. No more entity slots available."),
        };
        world
            .add_component(parent_monkey, LabelComponent { label : "Parent monkey".to_string() }).unwrap()
            .add_component(parent_monkey, TransformComponent {
                position: Vec3f { x: 0.0, y: 0.0, z: 0.0 }, rotation: QUATF_IDENTITY, scale: ONE_VEC3F * 0.3
            }).unwrap()
            .add_component(parent_monkey, RenderComponentInstanced {
                ids: indices.clone().into_iter().min().unwrap()..indices.clone().into_iter().max().unwrap() + 1,
                model: res_manager.load::<ModelResource>("models/monkey"),
                material: res_manager.load::<MaterialResource>("materials/unicolor")
            }).unwrap();



        // Create camera
        let camera = match world.create_entity() {
            Some(e) => e,
            None => throw!("Failed to create entity. No more entity slots available."),
        };
        world
            .add_component(camera, LabelComponent { label : "Camera".to_string() }).unwrap()
            .add_component(camera, TransformComponent {
                position: Vec3f { x: 0.0, y: 0.0, z: 0.0 }, rotation: QUATF_IDENTITY, scale: ONE_VEC3F
            }).unwrap();

        // Create camera uniform buffer
        let mut camera_buffer = Buffer::new(
            &render_instance,
            "Camera buffer",
            std::mem::size_of::<CameraUniform>(),
            wde_wgpu::BufferUsage::UNIFORM | wde_wgpu::BufferUsage::COPY_DST,
            None);
        let mut camera_uniform = CameraUniform::new();
        camera_uniform.world_to_screen = CameraUniform::get_world_to_screen(
            CameraComponent { aspect: 16.0 / 9.0, fovy: 60.0, znear: 0.1, zfar: 1000.0 },
            world.get_component::<TransformComponent>(camera).unwrap().clone()
        ).into();
        camera_buffer.write(&render_instance, bytemuck::cast_slice(&[camera_uniform]), 0);

        // End of world content
        drop(_world_content_span);


        // ======== RENDERER INITIALIZATION ========
        let _renderer_initialization_span = span!(Level::INFO, "renderer_initialization").entered();

        // Create renderer
        let renderer = Arc::new(RwLock::new(Renderer::new(
            &render_instance, &mut world, &mut res_manager, &mut camera_buffer
        ).await));

        // Update render
        renderer.write().unwrap().update(&render_instance, &world, &res_manager, &mut camera_buffer).await;
        renderer.write().unwrap().update_ssbo(&render_instance, &world, true);

        // End of renderer initialization
        drop(_renderer_initialization_span);
        
            

        // ======== MAIN LOOP ========
        let mut last_time = std::time::Instant::now();
        let mut fps_frames = vec![0.0; 20];
        let mut fps_frames_index = 0;
        let mut fps_avg = 0.0;
        
        // Create camera rotation
        let mut camera_rotation = Vec2f { x: 0.0, y: 0.0 };
        let camera_initial_rot = world.get_component::<TransformComponent>(camera).unwrap().rotation.clone();
        let move_speed = 0.1; // 10.0;
        let sensitivity = 0.05; // 8.0;
        
        // Run main loop
        loop {
            let _next_frame_span = span!(Level::INFO, "next_frame").entered();
            debug!("\n\n\n======== Next frame ========");
            
            // ====== Wait for next render event ======
            {
                let _next_frame_wait_span = span!(Level::INFO, "next_frame_wait").entered();
                trace!("Waiting for next frame.");
                let _ = event_relay_t.send(());

                // Check if should resize
                if let Ok(ev) = resize_r.try_recv() {
                    trace!("Handling resize event due to window event.");
                    
                    // Make sure to get the last resize event
                    let mut ev = ev;
                    while let Ok(ev_) = resize_r.try_recv() {
                        ev = ev_;
                    }

                    // Resize window
                    let (width, height) = ev;
                    window.resize(width, height);

                    // Resize render instance
                    render_instance.resize(width, height).unwrap_or_else(|e| {
                        throw!("Failed to resize render instance : {:?}.", e);
                    });

                    // Resize render
                    renderer.write().unwrap().resize(&render_instance, width, height).await;
                }
                
                // Wait for next event
                let ev = event_r.recv().await;
                if ev.is_none() { break; }
                match ev.unwrap() {
                    LoopEvent::Close => { break; },
                    LoopEvent::Redraw => { },
                    _ => { }
                }


                trace!("Handling next frame.");
            }


            // ====== Update world ======
            {
                let _world_update_span = span!(Level::INFO, "world_update").entered();

                // Handle inputs
                while let Ok(input) = input_r.try_recv() {
                    let key = input.physical_key;
                    let pressed = input.state == ElementState::Pressed;

                    // Set key state
                    keys_states.insert(key, pressed);
                }

                // Update world
                res_manager.update(&render_instance);

                // Update render
                renderer.write().unwrap().update(&render_instance, &world, &res_manager, &mut camera_buffer).await;
                renderer.write().unwrap().update_ssbo(&render_instance, &world, false);

                // Update camera
                {
                    // Update the camera controller
                    {
                        let mut transform = world.get_component::<TransformComponent>(camera).unwrap().clone();

                        // Update the transform position
                        let dt = ((last_time.elapsed().as_nanos()) as f64 / 1_000_000.0) as f32;
                        let forward = TransformComponent::forward(transform);
                        let right = TransformComponent::right(transform);
                        let up = TransformComponent::up(transform);
                        if *keys_states.get(&PhysicalKey::Code(KeyCode::KeyW)).unwrap_or(&false) {
                            transform.position += forward * move_speed * dt;
                        }
                        if *keys_states.get(&PhysicalKey::Code(KeyCode::KeyS)).unwrap_or(&false) {
                            transform.position -= forward * move_speed * dt;
                        }
                        if *keys_states.get(&PhysicalKey::Code(KeyCode::KeyD)).unwrap_or(&false) {
                            transform.position += right * move_speed * dt;
                        }
                        if *keys_states.get(&PhysicalKey::Code(KeyCode::KeyA)).unwrap_or(&false) {
                            transform.position -= right * move_speed * dt;
                        }
                        if *keys_states.get(&PhysicalKey::Code(KeyCode::KeyE)).unwrap_or(&false) {
                            transform.position += up * move_speed * dt;
                        }
                        if *keys_states.get(&PhysicalKey::Code(KeyCode::KeyQ)).unwrap_or(&false) {
                            transform.position -= up * move_speed * dt;
                        }

                        // Get x and y rotation
                        camera_rotation.x += if *keys_states.get(&PhysicalKey::Code(KeyCode::ArrowLeft)).unwrap_or(&false) {
                            sensitivity * dt
                        } else if *keys_states.get(&PhysicalKey::Code(KeyCode::ArrowRight)).unwrap_or(&false) {
                            -sensitivity * dt
                        } else {
                            0.0
                        };
                        camera_rotation.y += if *keys_states.get(&PhysicalKey::Code(KeyCode::ArrowUp)).unwrap_or(&false) {
                            sensitivity * dt
                        } else if *keys_states.get(&PhysicalKey::Code(KeyCode::ArrowDown)).unwrap_or(&false) {
                            -sensitivity * dt
                        } else {
                            0.0
                        };

                        // Clamp the y rotation (to avoid gimbal lock)
                        let bounds_rot_y = Rad(88.0);
                        camera_rotation.y = camera_rotation.y.clamp(-bounds_rot_y.0, bounds_rot_y.0);

                        // Update the transform rotation
                        let rot_x = Quatf::from_axis_angle(Vector3 {x:0.0, y:1.0, z:0.0}, Rad(camera_rotation.x));
                        let rot_y = Quatf::from_axis_angle(Vector3 {x:-1.0, y:0.0, z:0.0}, Rad(-camera_rotation.y));
                        transform.rotation = camera_initial_rot * rot_x * rot_y;

                        // Update the transform
                        world.set_component(camera, transform).unwrap();
                    }

                    // Update the uniform buffer
                    {
                        let mut camera_uniform = CameraUniform::new();
                        let surface_config = render_instance.surface_config.as_ref().unwrap();
                        camera_uniform.world_to_screen = CameraUniform::get_world_to_screen(
                            CameraComponent {
                                aspect: surface_config.width as f32 / surface_config.height as f32,
                                fovy: 60.0, znear: 0.1, zfar: 1000.0
                            },
                            world.get_component::<TransformComponent>(camera).unwrap().clone()
                        ).into();
                        camera_buffer.write(&render_instance, bytemuck::cast_slice(&[camera_uniform]), 0);
                    }
                }
            }


            // ====== Render ======
            {
                let mut should_resize = false;
                match renderer.read().unwrap().render(&render_instance, &world, &res_manager).await {
                    RenderEvent::Redraw(_) => {},
                    RenderEvent::Close => {
                        info!("Closing engine.");
                        break;
                    },
                    RenderEvent::Resize(_, _) => {
                        trace!("Handling resize event after querying texture.");
                        should_resize = true;
                    },
                    RenderEvent::None => {},
                }

                if should_resize {
                    // Resize render instance
                    render_instance.resize(window.size.0, window.size.1).unwrap_or_else(|e| {
                        throw!("Failed to resize render instance : {:?}.", e);
                    });

                    // Resize render
                    renderer.write().unwrap().resize(&render_instance, window.size.0, window.size.1).await;
                }
            }

            // Clear the receiver channel
            {
                let _clear_receiver_span = span!(Level::INFO, "clear_receiver").entered();
                while let Ok(_) = event_r.try_recv() {}
            }

            {
                // Calculate fps
                let fps = 1.0 / ((last_time.elapsed().as_nanos() as f64 / 1_000_000_000.0) as f32);
                fps_frames[fps_frames_index] = fps;
                fps_frames_index = fps_frames_index + 1;
                if fps_frames_index >= fps_frames.len() {
                    fps_frames_index = 0;
                    fps_avg = fps_frames.iter().sum::<f32>() / fps_frames.len() as f32;

                    // In release mode, print fps only now
                    if !cfg!(debug_assertions) {
                        info!("FPS: {:.2}", fps_avg);
                    }
                }

                // Set the last time
                last_time = std::time::Instant::now();

                // Print fps every time in debug mode
                if cfg!(debug_assertions) {
                    info!("FPS: {:.2}", fps_avg);
                }
            }
        }

        // End
        debug!("\n\n\n======== Ending engine ========");

        // Join window thread
        info!("Joining window thread.");
        {
            let _window_join_span = span!(Level::INFO, "window_join").entered();
            window_join.join().unwrap();
        }
        
        App {}
    }
}