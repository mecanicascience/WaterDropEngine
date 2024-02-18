#![allow(dead_code)]

use crate::widgets::Widget;

pub struct PropertiesWidget {
    pub value: f32,
    pub text: String,
}

impl PropertiesWidget {
    pub fn new() -> Self {
        Self {
            value: 5.0,
            text: String::from("Hello, World!"),
        }
    }
}

impl Widget for PropertiesWidget {
    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.label(format!("Test: {}", self.value));
        ui.text_edit_multiline(&mut self.text);
    }
}
