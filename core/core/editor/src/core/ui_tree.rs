#![allow(dead_code)]

use egui_dock::DockState;
use egui_dock::TabViewer;
use tracing::debug;
use tracing::info;

use crate::widgets::Widget;
use crate::widgets::PropertiesWidget;

pub struct UITree {
    widgets: Vec<Box<dyn Widget>>,
}

impl UITree {
    pub fn new() -> Self {
        info!("Creating UI tree widgets.");

        // Create widgets
        let widgets: Vec<Box<dyn Widget>> = vec![
            Box::new(PropertiesWidget::new()),
        ];

        Self {
            widgets,
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
                // let available_size = ui.available_size();
                // let tex_aspect_ratio = data.surface_config.width as f32 / data.surface_config.height as f32;
                // let cont_aspect_ratio = available_size.x as f32 / available_size.y as f32;
                // let (scale_w, scale_h) = if tex_aspect_ratio > cont_aspect_ratio {
                //     (available_size.x as f32, available_size.x as f32 / tex_aspect_ratio)
                // }
                // else {
                //     (available_size.y as f32 * tex_aspect_ratio, available_size.y as f32)
                // };
                // let tex_x = (available_size.x - scale_w) / 2.0;
                // let tex_y = (available_size.y - scale_h) / 2.0;

                // ui.allocate_space(egui::Vec2::new(tex_x, tex_y));
                // ui.image(self.render_texture_id, [scale_w, scale_h]);
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
