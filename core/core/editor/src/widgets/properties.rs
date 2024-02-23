#![allow(dead_code)]

use egui::{Color32, RichText, ScrollArea, TextEdit, TextStyle};
use tracing::debug;

use crate::widgets::Widget;
use wde_ecs::{CameraComponent, EntityIndex, LabelComponent, RenderComponent, RenderComponentInstanced, World};

pub struct PropertiesWidget {
    selected_entity: Option<EntityIndex>,
}

impl PropertiesWidget {
    pub fn new() -> Self {
        Self {
            selected_entity: None,
        }
    }

    pub fn format_entity_label(entity: EntityIndex, world: &World) -> String {
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
        
        if icon.is_empty() {
            icon = "📦".to_string();
        }

        // Return the formatted name
        format!("{} {}", icon, world.get_component::<LabelComponent>(entity).unwrap().label)
    }
}

impl Widget for PropertiesWidget {
    fn ui(&mut self, ui: &mut egui::Ui, world: &mut World) {
        debug!("Rendering UI for properties widget.");

        // Show the entity hierarchy
        let text_style = TextStyle::Body;
        let row_height = ui.text_style_height(&text_style);
        let entities = &world.get_entities_with_component::<LabelComponent>();
        ScrollArea::vertical().max_height(ui.available_height()*0.3).auto_shrink([false, false]).show_rows(ui,
            row_height,
            entities.len(),
            |ui, row_range| {
                for i in row_range {
                    ui.horizontal(|ui| {
                    ui.add_space(10.0);
                    ui.selectable_value(
                        &mut self.selected_entity,
                        Some(entities[i]),
                        RichText::new(PropertiesWidget::format_entity_label(entities[i], world)).color(Color32::WHITE));
                    });
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
                        world.add_component::<LabelComponent>(entity, LabelComponent { label: label_cloned });
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
            // ui.horizontal(|ui| {
            //     let mut noComponent = false;
            //     let mut old_static = if world.get_component::<RenderComponent>(entity).is_some() {
            //         EntityTypes::Dynamic
            //     } else if world.get_component::<RenderStaticComponent>(entity).is_some() {
            //         EntityTypes::Static
            //     } else {
            //         noComponent = true;
            //         EntityTypes::Dynamic
            //     };
            //     if !noComponent {
            //         let object_type = old_static;
            //         ui.add_space(ui.available_width() / 2.0 - 10.0 -" Dynamic ".len() as f32 * 7.0 - 5.0);
            //         ui.selectable_value(&mut old_static, EntityTypes::Dynamic, " Dynamic ");
            //         ui.add_space(10.0);
            //         ui.selectable_value(&mut old_static, EntityTypes::Static, " Static ");
            //         if old_static != object_type {
            //             match old_static {
            //                 EntityTypes::Dynamic => {
            //                     let model = world.get_component::<RenderStaticComponent>(entity).unwrap().model.clone();
            //                     world.remove_component::<RenderStaticComponent>(entity);
            //                     world.add_component::<RenderComponent>(entity, RenderComponent::new(model, "materials/normal.mat".to_string()));
            //                 },
            //                 EntityTypes::Static => {
            //                     let model = world.get_component::<RenderComponent>(entity).unwrap().model.clone();
            //                     world.remove_component::<RenderComponent>(entity);
            //                     world.add_component::<RenderStaticComponent>(entity, RenderStaticComponent::new(model, "materials/normal.mat".to_string()));

            //                     // Say to update SSBO
            //                     world.get_system_mut::<RenderSystem>().unwrap().static_entities_queue.push(entity);
            //                 }
            //             }
            //         }
            //     }
            // });
            ui.add_space(5.0);

            // Update properties
            // if self.selected_entity != self.last_selected_id {
            //     if let Some(transform) = world.get_component::<TransformComponent>(entity) {
            //         self.rotation_quat = transform.rotation;
            //         self.rotation_euler = transform.rotation.into();
            //     }
            //     else {
            //         self.rotation_quat = QUAT_IDENTITY;
            //         self.rotation_euler = Euler::new(Rad(0.0), Rad(0.0), Rad(0.0));
            //     }
            // }

            // Handle Component click
            // let module_click = |res: Option<CollapsingResponse<()>>, name, world: &mut World| {
            //     let res = match res {
            //         Some(res) => res,
            //         None => return
            //     };

            //     // On right click
            //     res.header_response.context_menu(|ui| {
            //         if ui.button("Remove Component").clicked() {
            //             match name {
            //                 "Render" => {
            //                     if world.get_component::<RenderComponent>(entity).is_some() {
            //                         world.remove_component::<RenderComponent>(entity);
            //                     }
            //                     else if world.get_component::<RenderStaticComponent>(entity).is_some() {
            //                         world.remove_component::<RenderStaticComponent>(entity);
            //                     }
            //                 },
            //                 "Camera" => { world.remove_component::<CameraComponent>(entity); },
            //                 _ => {}
            //             };
            //             ui.close_menu();
            //         }
            //     });
            // };


            // Transform Component
        //     ui.add_space(5.0);
        //     self.show_transform(ui, entity, world);

        //     // Other modules
        //     ui.add_space(5.0);
        //     module_click(self.show_camera(ui, entity, world), "Camera", world);
        //     ui.add_space(5.0);
        //     module_click(self.show_render(ui, entity, world, resource_manager, messages), "Render", world);

        //     // Add Component
        //     ui.add_space(5.0);
        //     ui.separator();
        //     ui.add_space(5.0);
        //     ui.horizontal(|ui| {
        //         ui.add_space(ui.available_width() / 2.0 - 5.0 -" Add Component None     ".len() as f32 * 7.0 - 5.0);
        //         egui::ComboBox::from_label("")
        //             .selected_text(format!("{:?}", self.selected_module))
        //             .show_ui(ui, |ui| {
        //                 ui.selectable_value(&mut self.selected_module, ModuleNames::None, "None     ");
        //                 if !world.get_component::<CameraComponent>(entity).is_some() {
        //                     ui.selectable_value(&mut self.selected_module, ModuleNames::Camera, "Camera");
        //                 }
        //                 if !world.get_component::<RenderComponent>(entity).is_some() {
        //                     ui.selectable_value(&mut self.selected_module, ModuleNames::Render, "Render");
        //                 }
        //             }
        //         );
        //         if ui.button("Add to entity").clicked() {
        //             match self.selected_module {
        //                 ModuleNames::Camera => {
        //                     world.add_component(entity, CameraComponent::new(
        //                         &data,
        //                         world.get_component::<TransformComponent>(entity).unwrap().clone()
        //                     ));
        //                 },
        //                 ModuleNames::Render => {
        //                     world.add_component(entity, RenderComponent::new("".to_string(), "materials/normal.mat".to_string()));
        //                 },
        //                 _ => {}
        //             }
        //         }

        //         ui.add_space(25.0);
        //         if ui.button(" New Entity ").clicked() {
        //             let entity = world.entity_manager.create_entity();
        //             world.add_component::<NameComponent>(entity, NameComponent { name : "New Entity".to_string() });
        //             world.add_component::<TransformComponent>(entity, TransformComponent::new(
        //                 ZERO_VEC3, QUAT_IDENTITY, ONE_VEC3
        //             ));
        //             self.selected_entity = Some(entity);
        //         }
        //     });
        }

        // Show resource selector
        // self.show_resource_selector(ui, resource_manager);

        // Update the selected entity
        // self.last_selected_id = self.selected_entity;
    }
}
