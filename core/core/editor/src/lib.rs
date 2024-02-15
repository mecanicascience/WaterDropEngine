mod editor;
mod editor_disabled;

#[cfg(feature = "editor")]
pub use editor::Editor;
#[cfg(not(feature = "editor"))]
pub use editor_disabled::Editor as Editor;
