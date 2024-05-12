use egui::{FontFamily, FontId, TextStyle};
use egui::Context;
use egui_dock::{DockArea, DockState, Style};
use tracing::info;
use wde_ecs::World;
use wde_resources::ResourcesManager;
use wde_wgpu::{CommandBuffer, RenderInstance, RenderTexture, Texture, TextureUsages};

use crate::{EditorPass, Plateform, ScreenDescriptor, UITree};

pub struct Editor {
    context: Context,
    plateform: Plateform,
    tree: UITree,
    tree_state: DockState<String>,

    render_pass: EditorPass,
    render_tex: Texture,
    render_to_texture_id: egui::TextureId
}

impl Editor {
    /// Create a new editor instance.
    /// 
    /// # Arguments
    /// 
    /// * `window_size` - The size of the window.
    /// * `instance` - The render instance.
    /// * `world` - The world.
    /// * `res_manager` - The resources manager.
    pub fn new(window_size: (u32, u32), instance: &RenderInstance<'_>, world: &mut World, res_manager: &mut ResourcesManager) -> Self {
        info!("Creating editor instance.");

        // Create egui context
        let context = Context::default();

        // Create egui plateform
        let plateform = Plateform::new(window_size);

        // Create egui render pass
        let egui_shader = std::fs::read_to_string("res/shaders/editor/editor_core.wgsl").unwrap_or_else(|_| {
            panic!("Failed to read editor shader.");
        });
        let mut render_pass = EditorPass::new(&instance.device,
            instance.surface_config.as_ref().unwrap().format,
            1, egui_shader.as_str());

        // Create render to texture UI
        let render_tex = Texture::new(instance,
            "Render to Texture UI",
            (instance.surface_config.as_ref().unwrap().width, instance.surface_config.as_ref().unwrap().height),
            Texture::SWAPCHAIN_FORMAT,
            TextureUsages::TEXTURE_BINDING);
        let render_to_texture_id = render_pass.egui_texture_from_wgpu_texture(
                &instance.device,
                &render_tex.view,
                wgpu::FilterMode::Linear);

        // Create tree
        let aspect_ratio = window_size.0 as f32 / window_size.1 as f32;
        let mut tree = UITree::new(render_to_texture_id, aspect_ratio);
        let tree_state = tree.init(world, res_manager);

        Editor {
            context,
            plateform,
            tree,
            tree_state,

            render_pass,
            render_tex,
            render_to_texture_id
        }
    }

    /// Render the editor.
    /// 
    /// # Arguments
    /// 
    /// * `instance` - The render instance.
    /// * `texture` - The render texture.
    /// 
    /// # Returns
    /// 
    /// True if the editor should close.
    pub fn render(&mut self, instance: &RenderInstance<'_>, texture: &RenderTexture) -> bool {
        let mut should_close = false;
        
        // Begin frame
        self.context.begin_frame(self.plateform.get_raw_input());

        // Set UI style
        let text_color = egui::Color32::from_rgba_premultiplied(255, 255, 255, 1);
        let mut style: egui::Style = (*self.context.style()).clone();
        style.visuals.widgets.noninteractive.fg_stroke = egui::Stroke::new(0.0, text_color);
        style.visuals.widgets.active.fg_stroke = egui::Stroke::new(0.0, text_color);
        style.visuals.override_text_color = Some(text_color);
        style.visuals.collapsing_header_frame = true;
        style.text_styles = [
            (TextStyle::Small, FontId::new(9.0, FontFamily::Proportional)),
            (TextStyle::Body, FontId::new(13.5, FontFamily::Proportional)),
            (TextStyle::Monospace, FontId::new(13.5, FontFamily::Monospace)),
            (TextStyle::Button, FontId::new(13.5, FontFamily::Proportional)),
            (TextStyle::Heading, FontId::new(18.0, FontFamily::Proportional)),
        ].into();
        self.context.set_style(style);

        // Record draw calls
        {
            // Debug ui
            // self.context.set_debug_on_hover(true);
            // egui::Window::new("🔧 Settings")
            //     .open(&mut open)
            //     .show(&self.context, |ui| {
            //         self.context.settings_ui(ui);
            //     });
                
            // Draw the tree
            egui::CentralPanel::default()
                .frame(egui::Frame::central_panel(&self.context.style()).inner_margin(0.))
                .show(&self.context, |ui| {
                    // Show menu
                    egui::menu::bar(ui, |ui| {
                        // File menu
                        ui.menu_button("    File    ", |ui| {
                            if ui.button("  Exit  ").clicked() {
                                should_close = true;
                            }
                        });

                        // Show tabs
                        ui.menu_button("  Window  ", |ui| {
                            for name in ["Editor", "Properties", "Resources", "World"] {
                                if ui.button(format!("  \"{}\" menu ", name)).clicked() {
                                    let t = self.tree_state.find_tab(&name.to_owned());
                                    match t {
                                        Some(t) => {
                                            self.tree_state.set_active_tab(t);
                                        },
                                        None => {
                                            self.tree_state.push_to_focused_leaf(name.to_owned());
                                        }
                                    }
                                    ui.close_menu();
                                }
                            }
                        });
                    });

                    // Show tabs
                    DockArea::new(&mut self.tree_state)
                        .tab_context_menus(false)
                        .style(Style::from_egui(self.context.style().as_ref()))
                        .show_inside(ui, &mut self.tree);
                });
        }

        // Tesselate to primitives
        let full_output = self.context.end_frame();
        let primitives = self.context.tessellate(full_output.shapes, full_output.pixels_per_point);

        // Copy render to texture UI
        let tex_size = instance.surface_config.as_ref().unwrap();
        self.render_tex.copy_from_texture(
            &instance,
            &texture.texture.texture,
            (tex_size.width, tex_size.height));

        // Render editor
        let tdelta = full_output.textures_delta;
        {
            // Create command buffer
            let mut command = CommandBuffer::new(instance, "Egui Draw");

            // Update render to texture
            self.render_pass.update_egui_texture_from_wgpu_texture(
                &instance.device, &self.render_tex.view,
                wgpu::FilterMode::Linear, self.render_to_texture_id).unwrap();

            // Upload all resources for the GPU.
            let screen_descriptor = ScreenDescriptor {
                physical_width: instance.surface_config.as_ref().unwrap().width,
                physical_height: instance.surface_config.as_ref().unwrap().height,
                scale_factor: 1.0,
            };
            self.render_pass
                .add_textures(&instance.device, &instance.queue, &tdelta)
                .expect("Failed to add textures.");
            self.render_pass.update_buffers(&instance.device, &instance.queue, &primitives, &screen_descriptor);

            // Record all render passes.
            self.render_pass
                .execute(
                    &mut command.encoder(),
                    &texture.view,
                    &primitives,
                    &screen_descriptor,
                    None
                )
                .unwrap();

            // Submit the commands
            command.submit(&instance);
        }

        // Remove all resources from the GPU unnecessary for the next frame.
        {
            self.render_pass
                .remove_textures(tdelta)
                .expect("Failed to remove textures.");
        }

        // Should not close
        should_close
    }

    /// Handle a resize event.
    /// 
    /// # Arguments
    /// 
    /// * `instance` - The render instance.
    /// * `size` - The new size of the window.
    pub fn handle_resize(&mut self, instance: &RenderInstance<'_>, size: (u32, u32)) {
        // Update plateform
        self.plateform.handle_resize(size);

        // Recreate render to texture
        self.render_tex = Texture::new(instance,
            "Render to Texture UI",
            (size.0, size.1),
            Texture::SWAPCHAIN_FORMAT,
            TextureUsages::TEXTURE_BINDING);

        // Update tree aspect ratio
        self.tree.aspect_ratio = size.0 as f32 / size.1 as f32;
    }

    /// Handle a mouse event.
    /// 
    /// # Arguments
    /// 
    /// * `event` - The mouse event.
    pub fn handle_mouse_event(&mut self, event: &winit::event::Event<()>) {
        self.plateform.handle_mouse_event(event);
    }

    /// Handle a window event.
    /// 
    /// # Arguments
    /// 
    /// * `event` - The window event.
    pub fn handle_input_event(&mut self, event: &winit::event::WindowEvent) {
        self.plateform.handle_input_event(event);
    }

    /// Returns true if the editor captures the event.
    /// 
    /// # Arguments
    /// 
    /// * `event` - The window event.
    /// 
    /// # Returns
    /// 
    /// True if the editor captures the event.
    pub fn captures_event(&self, event: &winit::event::WindowEvent) -> bool {
        self.plateform.captures_event(event, &self.context)
    }
}
