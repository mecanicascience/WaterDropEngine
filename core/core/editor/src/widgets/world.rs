#![cfg(feature = "editor")]

use egui::{CollapsingHeader, Color32, RichText, TextStyle, Ui};
use egui_extras::{Column, TableBuilder};
use tracing::debug;
use wde_ecs::{empty_signature, LabelComponent, World};
use wde_resources::ResourcesManager;

use crate::Widget;

pub struct WorldWidget;

impl WorldWidget {
    pub fn new() -> Self where Self: Sized {
        Self {}
    }

        /// Display informations about the components in the world.
    /// 
    /// # Arguments
    /// 
    /// * `ui` - The egui Ui.
    /// * `world` - The world.
    fn world_components(&mut self, ui: &mut Ui, world: &mut World) {
        let text_style = TextStyle::Body;
        let row_height = ui.text_style_height(&text_style) * 1.3;
        TableBuilder::new(ui)
            .column(Column::auto().resizable(true).at_least("1000".len() as f32 * 7.0 + 5.0).clip(true))
            .column(Column::initial("Long label name".len() as f32 * 7.0 * 6.0 + 30.0).resizable(true).clip(true))
            .column(Column::remainder())
            .header(20.0, |mut header| {
                header.col(|ui| {
                    ui.strong(" Component ");
                });
                header.col(|ui| {
                    ui.strong(" Name ");
                });
                header.col(|ui| {
                    ui.strong(" Signature ");
                });
            })
            .body(|body| {
                body.rows(
                row_height,
                world.component_manager.component_type_count as usize,
                |mut row| {
                    let row_index = row.index();

                    // Get the component
                    let comp_type_id = world.component_manager.component_types_list[row_index as usize];
                    let comp_index = world.component_manager.component_types.get(&comp_type_id).unwrap();
                    let comp_name = &world.component_manager.component_names[*comp_index as usize];

                    // Components
                    row.col(|ui| {
                        ui.add_space(5.0);
                        ui.label(format!("{}", comp_index));
                    });

                    // Icon and name
                    row.col(|ui| {
                        ui.add_space(5.0);
                        
                        // Show the entity ui
                        ui.horizontal(|ui| {
                            ui.add_space(10.0);
                            ui.label(format!("{:?}", comp_name));
                        });
                    });

                    // Modules
                    row.col(|ui| {
                        ui.add_space(5.0);
                        let mut signature = empty_signature();
                        signature.set(*comp_index, true);
                        ui.label(format!("{:?}", signature));
                    });
                });
            });

        ui.add_space(10.0);
    }

    /// Display informations about the entities in the world.
    /// 
    /// # Arguments
    /// 
    /// * `ui` - The egui Ui.
    /// * `world` - The world.
    fn world_entities(&mut self, ui: &mut Ui, world: &mut World) {
        let text_style = TextStyle::Body;
        let row_height = ui.text_style_height(&text_style) * 1.3;
        TableBuilder::new(ui)
            .column(Column::initial("100000".len() as f32 * 7.0 + 5.0).resizable(true).clip(true))
            .column(Column::initial("Long label name".len() as f32 * 7.0 * 6.0 + 30.0).resizable(true).clip(true))
            .column(Column::remainder())
            .header(20.0, |mut header| {
                header.col(|ui| {
                    ui.strong(" Entity ");
                });
                header.col(|ui| {
                    ui.strong(" Name ");
                });
                header.col(|ui| {
                    ui.strong(" Signature ");
                });
            })
            .body(|body| {
                body.rows(
                row_height,
                world.entity_manager.get_all_entities().len() as usize,
                |mut row| {
                    let row_index = row.index();

                    // Get the entity
                    let entity_index = world.entity_manager.get_all_entities()[row_index as usize];

                    // ID
                    row.col(|ui| {
                        ui.add_space(5.0);
                        ui.label(format!("{}", entity_index));
                    });

                    // Icon and name
                    row.col(|ui| {
                        ui.add_space(5.0);
                        
                        // Show the entity ui
                        ui.horizontal(|ui| {
                            ui.add_space(10.0);
                            ui.label(RichText::new(
                                format!("{}", world.get_component::<LabelComponent>(entity_index).unwrap().label)
                                ).color(Color32::WHITE));
                        });
                    });

                    // Modules
                    row.col(|ui| {
                        ui.add_space(5.0);
                        ui.label(format!("{:?}", world.entity_manager.get_signature(entity_index)));
                    });
                });
            });
    }
}

impl Widget for WorldWidget {
    fn ui(&mut self, ui: &mut egui::Ui, world: &mut wde_ecs::World, _res_manager: &mut ResourcesManager) {
        debug!("Rendering UI for world widget.");

        egui::TopBottomPanel::top("components_panel")
            .resizable(true)
            .min_height(60.0)
            .show_inside(ui, |ui| {
                CollapsingHeader::new("Components").default_open(true).show(ui, |ui| {
                    self.world_components(ui, world);
                });
            });

        egui::TopBottomPanel::top("entities_panel")
            .resizable(true)
            .default_height(180.0)
            .min_height(60.0)
            .show_inside(ui, |ui| {
                CollapsingHeader::new("Entities").default_open(true).show(ui, |ui| {
                    self.world_entities(ui, world);
                });
            });
    }
}
