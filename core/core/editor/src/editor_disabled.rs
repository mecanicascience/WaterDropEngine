#![cfg(not(feature = "editor"))]

pub struct Editor;

impl Editor {
    pub fn new() -> Self { Editor {} }
}
