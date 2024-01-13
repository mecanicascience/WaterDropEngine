use wde_game::*;
use wde_logger::*;

#[tokio::main]
async fn main() {
    // Create logger
    create_logger(LEVEL::TRACE);

    // Create app
    App::new().await;
}
