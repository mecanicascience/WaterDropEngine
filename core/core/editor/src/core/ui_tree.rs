#![allow(dead_code)]

use egui::Color32;
use egui::Rect;
use egui::Sense;
use egui_dock::DockState;
use egui_dock::TabViewer;
use tracing::debug;
use tracing::info;

use crate::widgets::Widget;
use crate::widgets::PropertiesWidget;

pub struct UITree {
    pub aspect_ratio: f32,
    widgets: Vec<Box<dyn Widget>>,
    render_texture_id: egui::TextureId,
}

impl UITree {
    pub fn new(render_texture_id: egui::TextureId, aspect_ratio: f32) -> Self {
        info!("Creating UI tree widgets.");

        // Create widgets
        let widgets: Vec<Box<dyn Widget>> = vec![
            Box::new(PropertiesWidget::new()),
        ];

        Self {
            widgets,
            render_texture_id,
            aspect_ratio
        }
    }

    pub fn init(&mut self) -> DockState<String> {
        let mut dock_state = DockState::new(vec![]);

        // Create the tree
        let tree = dock_state.main_surface_mut();
        tree.push_to_first_leaf("Editor".to_string());
        tree.split_right(tree.find_tab(&"Editor".to_string()).unwrap().0, 0.8, vec!["Properties".to_string()]);

        // Set active tab
        let active_tab = tree.find_tab(&"Editor".to_string()).unwrap();
        tree.set_active_tab(active_tab.0, active_tab.1);

        dock_state
    }
}

impl TabViewer for UITree {
    type Tab = String;

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        debug!("Rendering UI for widgets.");

        // Draw the widgets
        match tab.as_str() {
            "Editor" => {
                // Show render texture, limited by available size, preserving aspect ratio
                let available_size = ui.available_size();
                let cont_aspect_ratio = available_size.x as f32 / available_size.y as f32;
                let (scale_w, scale_h) = if self.aspect_ratio > cont_aspect_ratio {
                    (available_size.x as f32, available_size.x as f32 / self.aspect_ratio)
                }
                else {
                    (available_size.y as f32 * self.aspect_ratio, available_size.y as f32)
                };
                let tex_x = (available_size.x - scale_w) / 2.0;
                let tex_y = (available_size.y - scale_h) / 2.0;

                // Draw the render texture
                let (_res, painter) = ui.allocate_painter(
                    egui::Vec2::new(tex_x + scale_w, tex_y + scale_h), Sense::hover());
                painter.image(
                    self.render_texture_id,
                    Rect { min: egui::Pos2::new(tex_x, tex_y), max: egui::Pos2::new(tex_x + scale_w, tex_y + scale_h) },
                    Rect { min: egui::Pos2::ZERO, max: egui::Pos2::new(1.0, 1.0) },
                    Color32::WHITE
                );
            },
            "Properties" => {
                self.widgets[0].ui(ui);
            },
            _ => {
                ui.label("Unknown UI tab.");
            }
        }
    }

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        (&*tab).into()
    }
}
