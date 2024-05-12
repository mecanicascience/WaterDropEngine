use std::{collections::HashMap, rc::Rc, sync::RwLock};

use tokio::sync::mpsc;
#[cfg(feature = "editor")]
use wde_editor::Editor;
use wde_game::*;
use wde_logger::{debug, info, throw, trace, tracing::{span, Level}, Logger};
use wde_resources::ResourcesManager;
use wde_wgpu::{ElementState, Event, LoopEvent, PhysicalKey, RenderEvent, RenderInstance, Window, WindowEvent};

async fn run() {
    // Create logger
    let logger = Logger::new("log.txt", "trace.json");

    // Configure environment if on debug mode
    if cfg!(debug_assertions) {
        // Set the RUST_BACKTRACE environment variable to 1
        std::env::set_var("RUST_BACKTRACE", "0");
    }
    debug!("======== Starting engine ========");


    // ======== WINDOW CREATION ========
    let _window_creation_span = span!(Level::INFO, "window_init").entered();
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
    let (mouse_t, mut mouse_r) = mpsc::unbounded_channel();
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
                    event: WindowEvent::KeyboardInput { .. },
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

                // Handle mouse events
                Event::WindowEvent {
                    event: WindowEvent::CursorMoved { .. },
                    ..
                } => {
                    mouse_t.send(event).unwrap_or_else(|e| {
                        throw!("Failed to send mouse event : {}.", e);
                    });
                },
                Event::WindowEvent {
                    event: WindowEvent::MouseInput { .. },
                    ..
                } => {
                    mouse_t.send(event).unwrap_or_else(|e| {
                        throw!("Failed to send mouse event : {}.", e);
                    });
                },
                Event::WindowEvent {
                    event: WindowEvent::MouseWheel { .. },
                    ..
                } => {
                    mouse_t.send(event).unwrap_or_else(|e| {
                        throw!("Failed to send mouse event : {}.", e);
                    });
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
    let _engine_initialization_span = span!(Level::INFO, "engine_init").entered();

    // Create list of input keys
    let mut keys_states: HashMap<PhysicalKey, bool> = HashMap::new();

    // Create resource manager
    let mut res_manager = ResourcesManager::new();
    
    // Wait for window
    let window = window_r.recv().await.unwrap();
    let mut window_size = window.init_size;

    // Create render instance
    let mut render_instance = RenderInstance::new("WaterDropEngine", window).await;
    drop(_engine_initialization_span);



    // ======== SCENE INITIALIZATION ========
    let _scene_initialization_span = span!(Level::INFO, "scene_init").entered();
    let mut scene = Scene::new(&mut res_manager);

    // Create editor
    #[cfg(feature = "editor")]
    let mut editor = Editor::new(window_size, &render_instance, &mut scene.world, &mut res_manager);
    drop(_scene_initialization_span);



    // ======== RENDERER INITIALIZATION ========
    let _renderer_initialization_span = span!(Level::INFO, "renderer_init").entered();

    // Create renderer
    let renderer = Rc::new(RwLock::new(Renderer::new(
        &render_instance, &mut scene, &mut res_manager
    ).await));

    // End of renderer initialization
    drop(_renderer_initialization_span);
    
        

    // ======== MAIN LOOP ========
    let mut last_fps_time = std::time::Instant::now();
    let mut last_update_time = std::time::Instant::now();
    let mut fps_frames = [0.0; 40];
    let mut fps_frames_index = 0;
    let mut fps_avg = 0.0;
    let update_fps = 120.0;
    
    // Run main loop
    loop {
        let _next_frame_span = span!(Level::INFO, "next_frame").entered();
        debug!("\n\n\n======== Next frame ========");
        
        // ====== Handle window events ======
        let mut should_render = false;
        {
            let _next_frame_wait_span = span!(Level::INFO, "next_frame_wait").entered();
            trace!("Handling window events.");
            let _ = event_relay_t.send(());

            // Check for next render event
            if let Ok(ev) = event_r.try_recv() {
                match ev {
                    LoopEvent::Close => { break; },
                    LoopEvent::Redraw => { should_render = true; },
                    _ => { }
                }
            }

            // Wait for at least target update fps
            if !should_render {
                let elapsed_time = last_update_time.elapsed().as_nanos();
                let target_time = (1_000_000_000.0 / update_fps) as u128;
                if elapsed_time < target_time {
                    continue;
                }
            }
            last_update_time = std::time::Instant::now();

            // Check if should resize
            if let Ok(ev) = resize_r.try_recv() {
                trace!("Handling resize event due to window event.");
                
                // Make sure to get the last resize event
                let mut ev = ev;
                while let Ok(ev_) = resize_r.try_recv() {
                    ev = ev_;
                }

                // Send resize event to editor
                #[cfg(feature = "editor")]
                editor.handle_resize(&render_instance, ev);

                // Resize window
                let (width, height) = ev;
                window_size = (width, height);

                // Resize render instance
                render_instance.resize(width, height).unwrap_or_else(|e| {
                    throw!("Failed to resize render instance : {:?}.", e);
                });

                // Resize render
                renderer.write().unwrap().resize(&render_instance, width, height);
            }

            // Check for mouse events
            while let Ok(ev) = mouse_r.try_recv() {
                #[cfg(feature = "editor")]
                editor.handle_mouse_event(&ev);
            }
        }


        // ====== Update world ======
        {
            let _world_update_span = span!(Level::INFO, "world_update").entered();
            trace!("Updating world.");

            // Handle inputs
            while let Ok(input) = input_r.try_recv() {
                if let Event::WindowEvent { event, .. } = input {
                    // Update editor
                    #[cfg(feature = "editor")]
                    editor.handle_input_event(&event);

                    // Check if use input
                    #[cfg(feature = "editor")]
                    if !editor.captures_event(&event) {
                        // Handle keyboard input
                        if let WindowEvent::KeyboardInput { event, .. } = event {
                            let key = event.physical_key;
                            let pressed = event.state == ElementState::Pressed;

                            // Set key state
                            keys_states.insert(key, pressed);
                        }
                    }
                }
            }

            // Update resources manager (resources async loading and releasing)
            res_manager.update(&render_instance);

            // Update scene
            scene.set_keys_states(keys_states.clone());
            scene.update();

            // Update render
            renderer.write().unwrap().update(&render_instance, &scene, &res_manager);
        }


        // ====== Render ======
        if should_render {
            let _render_span = span!(Level::INFO, "render").entered();
            trace!("Rendering.");

            // Acquire render texture
            let mut should_resize = false;
            match RenderInstance::get_current_texture(&render_instance) {
                RenderEvent::Redraw(render_texture) => {
                    let mut should_close = false;

                    // Render world
                    renderer.read().unwrap().render(&render_instance, &scene, &mut res_manager, &render_texture);

                    // Render editor
                    #[cfg(feature = "editor")]
                    if editor.render(&render_instance, &render_texture) {
                        should_close = true;
                    }

                    // Present frame
                    let _ = render_instance.present(render_texture);

                    // Check if should close
                    if should_close {
                        info!("Closing engine.");
                        break;
                    }
                },
                RenderEvent::Close => {
                    info!("Closing engine.");
                    break;
                },
                RenderEvent::Resize(_, _) => {
                    should_resize = true;
                },
                RenderEvent::None => {},
            }

            // Resize
            if should_resize {
                debug!("Handling resize event due to render event.");

                // Resize render instance
                render_instance.resize(window_size.0, window_size.1).unwrap_or_else(|e| {
                    throw!("Failed to resize render instance : {:?}.", e);
                });

                // Resize render
                renderer.write().unwrap().resize(&render_instance, window_size.0, window_size.1);
            }
        }

        // Clear the render receiver channel
        {
            let _clear_receiver_span = span!(Level::INFO, "clear_receiver").entered();
            while event_r.try_recv().is_ok() { }
        }

        {
            // Calculate fps
            let fps = 1.0 / ((last_fps_time.elapsed().as_nanos() as f64 / 1_000_000_000.0) as f32);
            fps_frames[fps_frames_index] = fps;
            fps_frames_index += 1;
            if fps_frames_index >= fps_frames.len() {
                fps_frames_index = 0;
                fps_avg = fps_frames.iter().sum::<f32>() / fps_frames.len() as f32;

                // In release mode, print fps only now
                if !cfg!(debug_assertions) {
                    info!("FPS: {:.2}", fps_avg);
                }
            }

            // Set the last time
            last_fps_time = std::time::Instant::now();

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

    // Close logger
    logger.close();
}

#[tokio::main]
async fn main() {
    run().await;
}
