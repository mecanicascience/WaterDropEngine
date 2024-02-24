#![cfg(feature = "editor")]

use egui::{CollapsingHeader, CollapsingResponse, Color32, RichText, ScrollArea, TextEdit, TextStyle, Ui};
use egui_extras::{Column, TableBuilder};
use tracing::{debug, error};
use wde_math::{Euler, Quatf, Rad, ONE_VEC3F, QUATF_IDENTITY, ZERO_VEC3F};

use crate::{widgets::Widget, WidgetUtils};
use wde_ecs::{CameraComponent, EntityIndex, LabelComponent, RenderComponent, RenderComponentChild, RenderComponentInstanced, RenderComponentSSBODynamic, RenderComponentSSBOStatic, TransformComponent, World};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RenderEntityType {
    Static,
    Dynamic
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ModuleNames {
    None,
    Camera,
    Render,
    RenderInstanced
}

#[derive(Debug)]
pub struct PropertiesWidget {
    // Last selected entity
    selected_entity: Option<EntityIndex>,
    last_selected_entity: Option<EntityIndex>,
    selected_module: ModuleNames,

    // Visual sliders
    show_rotation_as_euler: bool,
    rotation_euler: Euler<Rad<f32>>,
    rotation_quat: Quatf,
}

impl PropertiesWidget {
    pub fn new() -> Self {
        Self {
            selected_entity: None,
            last_selected_entity: None,
            selected_module: ModuleNames::None,
            show_rotation_as_euler: true,
            rotation_euler: Euler::new(Rad(0.0), Rad(0.0), Rad(0.0)),
            rotation_quat: QUATF_IDENTITY,
        }
    }

    /// Format the label of a entity.
    ///
    /// # Arguments
    ///
    /// * `entity` - Entity to format the label of.
    /// * `world` - World to get the label from.
    pub fn format_entity_label(entity: EntityIndex, world: &World) -> Option<String> {
        // Get the icon
        let mut icon = String::new();
        if world.get_component::<RenderComponent>(entity).is_some() {
            icon += "🎨";
        }
        if world.get_component::<RenderComponentInstanced>(entity).is_some() {
            icon += "🎨🔢";
        }
        if world.get_component::<CameraComponent>(entity).is_some() {
            icon += "📷";
        }
        if world.get_component::<RenderComponentChild>(entity).is_some() {
            return None;
        }
        
        if icon.is_empty() {
            icon = "📦".to_string();
        }

        // Return the formatted name
        Some(format!("{} {}", icon, world.get_component::<LabelComponent>(entity).unwrap().label))
    }

    /// Show the transform Component of a entity.
    /// 
    /// # Arguments
    /// 
    /// * `ui` - Egui context.
    /// * `entity` - Entity to show the transform of.
    /// * `world` - World to get the transform from.
    fn show_transform(&mut self, ui: &mut Ui, entity: EntityIndex, world: &mut World) -> Option<CollapsingResponse<()>> {
        let mut transform = match world.get_component::<TransformComponent>(entity) {
            Some(transform) => transform.clone(),
            None => return None
        };

        let height = ui.text_style_height(&TextStyle::Body);
        let width = "Position".len() as f32 * 7.0 + 5.0; // Longest label + padding
        let header = Some(CollapsingHeader::new(RichText::new("📐 Transform").color(Color32::WHITE))
            .default_open(true)
            .show_background(true)
            .show(ui, |ui| {
                TableBuilder::new(ui)
                    .column(Column::auto().resizable(true).at_least(width).clip(true))
                    .column(Column::auto())
                    .column(Column::remainder())
                    .body(|mut body| {
                        body.row(height, |mut row| {
                            WidgetUtils::drag_vector(&mut row, &mut transform.position, "Position", None)
                        });
                        body.row(height, |mut row| {
                            // Rotation
                            if self.show_rotation_as_euler {
                                WidgetUtils::drag_vector_euler(&mut row, &mut self.rotation_euler, "Rotation", None);
                                transform.rotation = self.rotation_euler.into();

                                // Switch between euler and quaternion
                                row.col(|ui| {
                                    if ui.button("Q").clicked() {
                                        self.show_rotation_as_euler = false;
                                        self.rotation_quat = transform.rotation;
                                    }
                                });
                            }
                            else {
                                WidgetUtils::drag_vector_quat(&mut row, &mut self.rotation_quat, "Rotation", None);
                                transform.rotation = self.rotation_quat;

                                // Switch between euler and quaternion
                                row.col(|ui| {
                                    if ui.button("E").clicked() {
                                        self.show_rotation_as_euler = true;
                                        self.rotation_euler = transform.rotation.into();
                                    }
                                });
                            }
                        });
                        body.row(height, |mut row| {
                            WidgetUtils::drag_vector(&mut row, &mut transform.scale, "Scale", None)
                        });
                    });
                }));

        // Update the transform
        world.set_component::<TransformComponent>(entity, transform);

        return header;
    }


    /// Show the camera Component of a entity.
    /// 
    /// # Arguments
    /// 
    /// * `ui` - Egui context.
    /// * `entity` - Entity to show the camera of.
    /// * `world` - World to get the camera from.
    fn show_camera(&mut self, ui: &mut Ui, entity: EntityIndex, world: &mut World) -> Option<CollapsingResponse<()>> {
        let mut camera = match world.get_component::<CameraComponent>(entity) {
            Some(camera) => camera.clone(),
            None => return None
        };

        let height = ui.text_style_height(&TextStyle::Body);
        let width = "Near plane".len() as f32 * 7.0 + 5.0; // Longest label + padding
        let c = Color32::from_rgb(150, 150, 150);
        let header = Some(CollapsingHeader::new(RichText::new("📷 Camera").color(Color32::WHITE))
            .default_open(true)
            .show_background(true)
            .show(ui, |ui| {
                TableBuilder::new(ui)
                    .column(Column::auto().resizable(true).at_least(width).clip(true))
                    .column(Column::remainder())
                    .body(|mut body| {
                        body.row(height, |mut row| {
                            row.col(|ui| {
                                ui.label("Near plane");
                            });
                            row.col(|ui| {
                                ui.horizontal(|ui| {
                                    ui.add_space(10.0);
                                    WidgetUtils::drag_value(ui, &mut camera.znear, Some(0.1), Some(c), Some((0.01, 100000.0)));
                                });
                            });
                        });
                        body.row(height, |mut row| {
                            row.col(|ui| {
                                ui.label("Far plane");
                            });
                            row.col(|ui| {
                                ui.horizontal(|ui| {
                                    ui.add_space(10.0);
                                    WidgetUtils::drag_value(ui, &mut camera.zfar, Some(0.1), Some(c), Some((0.01, 100000.0)));
                                });
                            });
                        });
                        body.row(height, |mut row| {
                            row.col(|ui| {
                                ui.label("FOV");
                            });
                            row.col(|ui| {
                                ui.horizontal(|ui| {
                                    ui.add_space(10.0);
                                    WidgetUtils::drag_value(ui, &mut camera.fovy, Some(0.1), Some(c), None);
                                });
                            });
                        });
                    });
            }));

        // Update the camera
        world.set_component::<CameraComponent>(entity, camera);

        return header;
    }

    /// Show the render Component of a entity.
    /// 
    /// # Arguments
    /// 
    /// * `ui` - Egui context.
    /// * `entity` - Entity to show the render of.
    /// * `world` - World to get the render from.
    /// * `resourcesManager` - Resources manager to get the resources from.
    /// * `messages` - Messages to send to the engine.
    fn show_render(&mut self, ui: &mut Ui, entity: EntityIndex, world: &mut World) -> Option<CollapsingResponse<()>> {
        let is_instanced = world.get_component::<RenderComponentInstanced>(entity).is_some();
        let is_not_instanced = world.get_component::<RenderComponent>(entity).is_some();
        let render_comp = if is_instanced {
            let c = world.get_component::<RenderComponentInstanced>(entity).unwrap();
            (c.model.clone(), c.material.clone())
        } else if is_not_instanced {
            let c = world.get_component::<RenderComponent>(entity).unwrap();
            (c.model.clone(), c.material.clone())
        } else {
            return None;
        };
        
        let header = Some(CollapsingHeader::new(RichText::new("🎨 Render").color(Color32::WHITE))
            .default_open(true)
            .show_background(true)
            .show(ui, |ui| {
                let height = ui.text_style_height(&TextStyle::Body);
                let width = "Model ".len() as f32 * 7.0 + 9.0; // Longest label + padding
                ui.vertical(|ui| {
                    TableBuilder::new(ui)
                        .column(Column::auto().resizable(true).at_least(width).clip(true))
                        .column(Column::remainder())
                        .body(|mut body| {
                            body.row(height, |mut row| {
                                row.col(|ui| {
                                    ui.horizontal(|ui| {
                                        ui.add_space(5.0);
                                        ui.label("Model ");
                                    });
                                });
                                row.col(|ui| {
                                    ui.horizontal(|ui| {
                                        ui.add_space(5.0);
                                        ui.label(match render_comp.0 {
                                            Some(model) => model.label.clone(),
                                            None => "None".to_string()
                                        });
                                    });
                                });
                            });

                            body.row(height, |mut row| {
                                row.col(|ui| {
                                    ui.horizontal(|ui| {
                                        ui.add_space(5.0);
                                        ui.label("Material ");
                                    });
                                });
                                row.col(|ui| {
                                    ui.horizontal(|ui| {
                                        ui.add_space(5.0);
                                        ui.label(match render_comp.1 {
                                            Some(material) => material.label.clone(),
                                            None => "None".to_string()
                                        });
                                    });
                                });
                            });
                        });
                });
            }));
        return header;
    }
}

impl Widget for PropertiesWidget {
    #[tracing::instrument(skip(ui))]
    fn ui(&mut self, ui: &mut egui::Ui, world: &mut World) {
        debug!("Rendering UI for properties widget.");

        // Get the entity list
        let mut entity_list = world.get_entities_with_component::<RenderComponentInstanced>().clone();
        entity_list = entity_list.iter().chain(world.get_entities_with_component::<RenderComponent>().clone().iter()).cloned().collect::<Vec<EntityIndex>>();
        entity_list = entity_list.iter().chain(world.get_entities_with_component::<CameraComponent>().clone().iter()).cloned().collect::<Vec<EntityIndex>>();
        entity_list.sort();

        // Show the entity hierarchy
        let text_style = TextStyle::Body;
        let row_height = ui.text_style_height(&text_style);
        ScrollArea::vertical().max_height(ui.available_height()*0.3).auto_shrink([false, false]).show_rows(ui,
            row_height,
            entity_list.len(),
            |ui, row_range| {
                for i in row_range {
                    let index = entity_list[i];
                    match PropertiesWidget::format_entity_label(index, world) {
                        Some(prop) => {
                            ui.horizontal(|ui| {
                                ui.add_space(10.0);
                                ui.selectable_value(
                                    &mut self.selected_entity,
                                    Some(index),
                                    RichText::new(prop).color(Color32::WHITE));
                            });
                        },
                        None => {}
                    }
                }
            });

        ui.separator();

        // Show the properties of the selected entity
        if let Some(entity) = self.selected_entity {
            // Label and UUID
            ui.horizontal(|ui| {
                ui.label(" UUID:");
                ui.label(entity.to_string());

                ui.separator();

                ui.label("Label:");
                if let Some(label) = world.get_component::<LabelComponent>(entity) {
                    let mut label_cloned = label.clone().label;
                    ui.add_sized([ui.available_width() - 30.0, 20.0], TextEdit::singleline(&mut label_cloned));

                    // If the label has changed, update the component
                    if label.label != label_cloned {
                        world.set_component::<LabelComponent>(entity, LabelComponent { label: label_cloned });
                    }
                }
                else {
                    ui.label("None");
                }

                // Remove entity
                ui.add_space(5.0);
                if ui.button("X").clicked() {
                    if let Some(entity) = self.selected_entity {
                        world.destroy_entity(entity);
                        self.selected_entity = None;
                    }
                }
            });

            // Static or dynamic
            ui.add_space(5.0);
            ui.horizontal(|ui| {
                let mut no_component = false;
                let mut old_static = if world.get_component::<RenderComponentSSBODynamic>(entity).is_some() {
                    RenderEntityType::Dynamic
                } else if world.get_component::<RenderComponentSSBOStatic>(entity).is_some() {
                    RenderEntityType::Static
                } else {
                    no_component = true;
                    RenderEntityType::Dynamic
                };
                if !no_component {
                    let object_type = old_static;
                    ui.add_space(ui.available_width() / 2.0 - 10.0 -" Dynamic ".len() as f32 * 7.0 - 5.0);
                    ui.selectable_value(&mut old_static, RenderEntityType::Dynamic, " Dynamic ");
                    ui.add_space(10.0);
                    ui.selectable_value(&mut old_static, RenderEntityType::Static, " Static ");
                    if old_static != object_type {
                        match old_static {
                            RenderEntityType::Dynamic => {
                                let c = world.get_component::<RenderComponentSSBOStatic>(entity).unwrap().clone();
                                world.remove_component::<RenderComponentSSBOStatic>(entity);
                                world.add_component::<RenderComponentSSBODynamic>(entity, RenderComponentSSBODynamic { id: c.id });
                            },
                            RenderEntityType::Static => {
                                let c = world.get_component::<RenderComponentSSBODynamic>(entity).unwrap().clone();
                                world.remove_component::<RenderComponentSSBODynamic>(entity);
                                world.add_component::<RenderComponentSSBOStatic>(entity, RenderComponentSSBOStatic { id: c.id });
                            }
                        }
                    }
                }
            });
            ui.add_space(5.0);

            // Update properties
            if self.selected_entity != self.last_selected_entity {
                if let Some(transform) = world.get_component::<TransformComponent>(entity) {
                    self.rotation_quat = transform.rotation;
                    self.rotation_euler = transform.rotation.into();
                }
                else {
                    self.rotation_quat = QUATF_IDENTITY;
                    self.rotation_euler = Euler::new(Rad(0.0), Rad(0.0), Rad(0.0));
                }
            }

            // Handle Component click
            let module_click = |res: Option<CollapsingResponse<()>>, name, world: &mut World| {
                let res = match res {
                    Some(res) => res,
                    None => return
                };

                // On right click
                res.header_response.context_menu(|ui| {
                    if ui.button("Remove Component").clicked() {
                        match name {
                            "Render" => {
                                if world.get_component::<RenderComponent>(entity).is_some() {
                                    world.remove_component::<RenderComponent>(entity);
                                }
                                if world.get_component::<RenderComponentSSBOStatic>(entity).is_some() {
                                    world.remove_component::<RenderComponentSSBOStatic>(entity);
                                }
                                if world.get_component::<RenderComponentSSBODynamic>(entity).is_some() {
                                    world.remove_component::<RenderComponentSSBODynamic>(entity);
                                }
                                if world.get_component::<RenderComponentInstanced>(entity).is_some() {
                                    world.remove_component::<RenderComponentInstanced>(entity);
                                }
                                if world.get_component::<RenderComponentChild>(entity).is_some() {
                                    world.remove_component::<RenderComponentChild>(entity);
                                }
                            },
                            "Camera" => { world.remove_component::<CameraComponent>(entity); },
                            _ => {}
                        };
                        ui.close_menu();
                    }
                });
            };


            // Transform Component
            ui.add_space(5.0);
            self.show_transform(ui, entity, world);

            // Other modules
            ui.add_space(5.0);
            module_click(self.show_camera(ui, entity, world), "Camera", world);
            ui.add_space(5.0);
            module_click(self.show_render(ui, entity, world), "Render", world);

            // Add Component
            ui.add_space(5.0);
            ui.separator();
            ui.add_space(5.0);
            ui.horizontal(|ui| {
                ui.add_space(ui.available_width() / 2.0 - 5.0 -" Add Component None     ".len() as f32 * 7.0 - 5.0);
                egui::ComboBox::from_label("")
                    .selected_text(format!("{:?}", self.selected_module))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.selected_module, ModuleNames::None, "None     ");
                        if !world.get_component::<CameraComponent>(entity).is_some() {
                            ui.selectable_value(&mut self.selected_module, ModuleNames::Camera, "Camera");
                        }
                        if !world.get_component::<RenderComponent>(entity).is_some() {
                            ui.selectable_value(&mut self.selected_module, ModuleNames::Render, "Render");
                        }
                        if !world.get_component::<RenderComponentInstanced>(entity).is_some() {
                            ui.selectable_value(&mut self.selected_module, ModuleNames::RenderInstanced, "Render Instanced");
                        }
                    }
                );
                if ui.button("Add to entity").clicked() {
                    match self.selected_module {
                        ModuleNames::Camera => {
                            world.add_component(entity, CameraComponent { znear: 0.1, zfar: 1000.0, fovy: 60.0, aspect: 1.0 });
                        },
                        ModuleNames::Render => {
                            let render_index = world.get_next_render_index();
                            world.add_component(entity, RenderComponentSSBODynamic { id: render_index });
                            world.add_component(entity, RenderComponent { id: render_index, model: None, material: None });
                        },
                        ModuleNames::RenderInstanced => {
                            world.add_component(entity, RenderComponentInstanced { ids: 0..0, model: None, material: None });
                        },
                        _ => {}
                    }
                }

                ui.add_space(25.0);
                if ui.button(" New Entity ").clicked() {
                    let entity = world.create_entity();
                    if entity.is_none() {
                        error!("Failed to create entity.");
                        return;
                    }
                    world.add_component::<LabelComponent>(entity.unwrap(), LabelComponent { label : "New Entity".to_string() });
                    world.add_component::<TransformComponent>(entity.unwrap(), TransformComponent {
                        position: ZERO_VEC3F, rotation: QUATF_IDENTITY, scale: ONE_VEC3F
                    });
                    self.selected_entity = Some(entity.unwrap());
                }
            });
        }

        // Update the selected entity
        self.last_selected_entity = self.selected_entity;
    }
}
