use wde_game::*;
use wde_logger::Logger;

#[tokio::main]
async fn main() {
    // Create logger
    let logger = Logger::new("tracing.json");

    // Create app
    App::new().await;

    // Close logger
    logger.close();
}
