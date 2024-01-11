use wde_game::App;
use wde_logger::*;

#[tokio::main]
async fn main() {
    // Create logger
    create_logger(LEVEL::TRACE);

    // Create app
    App::new().await;
}
