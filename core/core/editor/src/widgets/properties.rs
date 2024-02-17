#![allow(dead_code)]

use crate::widgets::Widget;

pub struct PropertiesWidget {
    pub value: f32,
}

impl PropertiesWidget {
    pub fn new() -> Self {
        Self {
            value: 5.0,
        }
    }
}

impl Widget for PropertiesWidget {
    fn ui(&mut self, ui: &mut egui::Ui) {
        ui.label(format!("Test: {}", self.value));
    }
}
