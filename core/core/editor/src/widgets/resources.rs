#![cfg(feature = "editor")]

use egui::{CollapsingHeader, TextStyle};
use egui_extras::{Column, TableBuilder};
use tracing::debug;
use wde_ecs::World;
use wde_resources::{MaterialResource, ModelResource, ResourcesManager, ShaderResource, TextureResource};

use crate::Widget;

#[derive(Debug)]
pub struct ResourcesWidget;

impl ResourcesWidget {
    pub fn new() -> Self {
        Self {}
    }
}

impl Widget for ResourcesWidget {
    #[tracing::instrument(skip(ui))]
    fn ui(&mut self, ui: &mut egui::Ui, world: &mut World, res_manager: &mut ResourcesManager) {
        debug!("Rendering UI for resources widget.");
        let height = ui.text_style_height(&TextStyle::Body);
        let width = "Long label name".len() as f32 * 7.0 * 2.0 + 5.0; // Longest label + padding

        // Models resources
        CollapsingHeader::new("📦 Models").default_open(true).show(ui, |ui| {
            ui.horizontal(|ui| {
                TableBuilder::new(ui)
                    .column(Column::auto().resizable(true).at_least(width).clip(true))
                    .column(Column::remainder())
                    .body(|mut body| {
                        res_manager.get_res::<ModelResource>().iter().for_each(|res| {
                            if let Some(res) = res {
                                body.row(height, |mut row| {
                                    row.col(|ui| {
                                        ui.add_space(5.0);
                                        ui.label(format!("{}", res.label));
                                    });
                                    row.col(|ui| {
                                        ui.add_space(5.0);
                                        ui.label(format!("{}", res.path));
                                    });
                                });
                            }
                        });
                    });
            });
        });

        // Textures resources
        CollapsingHeader::new("🖼️ Textures").default_open(true).show(ui, |ui| {
            ui.horizontal(|ui| {
                TableBuilder::new(ui)
                    .column(Column::auto().resizable(true).at_least(width).clip(true))
                    .column(Column::remainder())
                    .body(|mut body| {
                        res_manager.get_res::<TextureResource>().iter().for_each(|res| {
                            if let Some(res) = res {
                                body.row(height, |mut row| {
                                    row.col(|ui| {
                                        ui.add_space(5.0);
                                        ui.label(format!("{}", res.label));
                                    });
                                    row.col(|ui| {
                                        ui.add_space(5.0);
                                        ui.label(format!("{}", res.path));
                                    });
                                });
                            }
                        });
                    });
            });
        });

        // Shaders resources
        CollapsingHeader::new("🎨 Shaders").default_open(true).show(ui, |ui| {
            ui.horizontal(|ui| {
                TableBuilder::new(ui)
                    .column(Column::auto().resizable(true).at_least(width).clip(true))
                    .column(Column::remainder())
                    .body(|mut body| {
                        res_manager.get_res::<ShaderResource>().iter().for_each(|res| {
                            if let Some(res) = res {
                                body.row(height, |mut row| {
                                    row.col(|ui| {
                                        ui.add_space(5.0);
                                        ui.label(format!("{}", res.label));
                                    });
                                    row.col(|ui| {
                                        ui.add_space(5.0);
                                        ui.label(format!("{}", res.path));
                                    });
                                });
                            }
                        });
                    });
            });
        });

        // Materials resources
        CollapsingHeader::new("🧱 Materials").default_open(true).show(ui, |ui| {
            ui.horizontal(|ui| {
                TableBuilder::new(ui)
                    .column(Column::auto().resizable(true).at_least(width).clip(true))
                    .column(Column::remainder())
                    .body(|mut body| {
                        res_manager.get_res::<MaterialResource>().iter().for_each(|res| {
                            if let Some(res) = res {
                                body.row(height, |mut row| {
                                    row.col(|ui| {
                                        ui.add_space(5.0);
                                        ui.label(format!("{}", res.label));
                                    });
                                });
                            }
                        });
                    });
            });
        });
    }
}
