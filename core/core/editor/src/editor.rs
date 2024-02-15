#![cfg(feature = "editor")]

use tracing::info;

pub struct Editor {

}

impl Editor {
    pub fn new() -> Self {
        info!("Editor enabled");

        Editor {}
    }
}
