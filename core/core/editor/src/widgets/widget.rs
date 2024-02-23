pub trait Widget {
    /// Draw the widget.
    /// 
    /// # Arguments
    /// 
    /// * `ui` - The UI to draw the widget on.
    /// * `world` - The world to draw the widget for.
    fn ui(&mut self, ui: &mut egui::Ui, world: &mut wde_ecs::World);
}
