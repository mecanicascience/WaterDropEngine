pub trait Widget {
    /// Draw the widget.
    /// 
    /// # Arguments
    /// 
    /// * `ui` - The UI to draw the widget on.
    fn ui(&mut self, ui: &mut egui::Ui);
}
