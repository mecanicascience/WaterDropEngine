use egui::{Color32, DragValue, Label, RichText};
use egui_extras::{Size, StripBuilder, TableRow};
use wde_math::{Euler, Quatf, Rad, Vec3f};

pub trait Widget {
    /// Draw the widget.
    /// 
    /// # Arguments
    /// 
    /// * `ui` - The UI to draw the widget on.
    /// * `world` - The world to draw the widget for.
    fn ui(&mut self, ui: &mut egui::Ui, world: &mut wde_ecs::World);
}


pub struct WidgetUtils;
impl WidgetUtils {
    /// Draw a vector with a label in a table and a 3 drag values.
    /// 
    /// # Arguments
    /// 
    /// * `row` - Row of the table.
    /// * `vec` - Vector to draw.
    /// * `label` - Label of the vector.
    /// * `speed` - Speed of the drag values (default: 0.01)
    pub fn drag_vector(row: &mut TableRow, vec: &mut Vec3f, label: &str, speed: Option<f32>) {
        row.col(|ui| {
            ui.label(label);
        });
        row.col(|ui| {
            let s = speed.unwrap_or(0.01);

            ui.horizontal(|ui| {
                ui.add_space(10.0);
                WidgetUtils::drag_value(ui, &mut vec.x, Some(s), Some(Color32::DARK_RED), None);
                WidgetUtils::drag_value(ui, &mut vec.y, Some(s), Some(Color32::DARK_GREEN), None);
                WidgetUtils::drag_value(ui, &mut vec.z, Some(s), Some(Color32::DARK_BLUE), None);
            });
        });
    }

    /// Draw a euler vector with a label in a table and a 3 drag values.
    /// 
    /// # Arguments
    /// 
    /// * `row` - Row of the table.
    /// * `vec` - Vector to draw.
    /// * `label` - Label of the vector.
    /// * `speed` - Speed of the drag values (default: 0.01)
    pub fn drag_vector_euler(row: &mut TableRow, vec: &mut Euler<Rad<f32>>, label: &str, speed: Option<f32>) {
        row.col(|ui| {
            ui.label(label);
        });
        row.col(|ui| {
            let s = speed.unwrap_or(0.01);

            ui.horizontal(|ui| {
                ui.add_space(10.0);
                WidgetUtils::drag_value(ui, &mut vec.x.0, Some(s), Some(Color32::DARK_RED), None);
                WidgetUtils::drag_value(ui, &mut vec.y.0, Some(s), Some(Color32::DARK_GREEN), None);
                WidgetUtils::drag_value(ui, &mut vec.z.0, Some(s), Some(Color32::DARK_BLUE), None);
            });
        });
    }

    /// Draw a quaternion with a label in a table and a 3 drag values.
    /// 
    /// # Arguments
    /// 
    /// * `row` - Row of the table.
    /// * `quat` - Quaternion to draw.
    /// * `label` - Label of the vector.
    /// * `speed` - Speed of the drag values (default: 0.01)
    pub fn drag_vector_quat(row: &mut TableRow, quat: &mut Quatf, label: &str, speed: Option<f32>) {
        row.col(|ui| {
            ui.label(label);
        });
        row.col(|ui| {
            let s = speed.unwrap_or(0.01);

            ui.horizontal(|ui| {
                ui.add_space(10.0);
                WidgetUtils::drag_value(ui, &mut quat.v.x, Some(s), Some(Color32::DARK_RED), None);
                WidgetUtils::drag_value(ui, &mut quat.v.y, Some(s), Some(Color32::DARK_GREEN), None);
                WidgetUtils::drag_value(ui, &mut quat.v.z, Some(s), Some(Color32::DARK_BLUE), None);
                WidgetUtils::drag_value(ui, &mut quat.s, Some(s), Some(Color32::DARK_GRAY), None);
            });
        });
    }

    /// Draw a custom drag value with a colored label.
    /// 
    /// # Arguments
    /// 
    /// * `ui` - Egui ui.
    /// * `value` - Value of the drag value.
    /// * `speed` - Speed of the drag value (default: 0.01)
    /// * `color` - Color of the label (default: white)
    pub fn drag_value(ui: &mut egui::Ui, value: &mut f32, speed: Option<f32>, color: Option<egui::Color32>, range: Option<(f32, f32)>) {
        let s = speed.unwrap_or(0.01);
        let c = color.unwrap_or(Color32::WHITE);

        ui.horizontal(|ui| {
            StripBuilder::new(ui)
                .size(Size::exact(0.05))
                .size(Size::exact(1.0))
                .horizontal(|mut strip| {
                    strip.cell(|ui| {
                        ui.add(Label::new(
                            RichText::new(" ").background_color(c)
                        ));
                    });
                    strip.cell(|ui| {
                        let mut dv = DragValue::new(value).speed(s)
                            .custom_formatter(|v, _| {
                                if v > 0.0 {
                                    format!("+{:.2}", v)
                                }
                                else if v == 0.0 {
                                    format!("+{:.2}", v.abs())
                                }
                                else {
                                    format!("{:.2}", v)
                                }
                            });
                        if let Some(r) = range {
                            dv = dv.clamp_range(r.0..=r.1);
                        }
                        ui.spacing_mut().item_spacing.x = 0.0;
                        ui.add(dv);
                    });
                });
            ui.add_space(5.0);
        });
    }
    

    /// Draw a text edit singleline with a label.
    /// 
    /// # Arguments
    /// 
    /// * `row` - Row of the table.
    /// * `label` - Label of the text.
    /// * `value` - Value of the text.
    pub fn text_edit_singleline(row: &mut TableRow, label: &str, value: &mut String) {
        row.col(|ui| {
            ui.label(label);
        });
        row.col(|ui| {
            ui.horizontal(|ui| {
                ui.add_space(3.0);
                ui.text_edit_singleline(value);
            });
        });
    }

    /// Draw a text value display with a label.
    /// 
    /// # Arguments
    /// 
    /// * `row` - Row of the table.
    /// * `label` - Label of the text.
    /// * `value` - Value of the text.
    pub fn text_single_line(row: &mut TableRow, label: &str, value: &str) {
        row.col(|ui| {
            ui.label(label);
        });
        row.col(|ui| {
            ui.horizontal(|ui| {
                ui.add_space(3.0);
                ui.label(RichText::new(value).background_color(as_egui(Vec3f { x: 0.11, y: 0.11, z: 0.11 })));
            });
        });
    }
}

/// Convert from egui::Color32 to Color
/// 
/// # Arguments
/// 
/// * `color` - The egui::Color32 to convert
fn as_egui(color: Vec3f) -> egui::Color32 {
    let c = 255.0 * color;
    egui::Color32::from_rgba_premultiplied(c.x as u8, c.y as u8, c.z as u8, (255.0 / 255.0) as u8)
}
