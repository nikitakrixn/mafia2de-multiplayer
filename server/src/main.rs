use common::logger;

fn main() {
    if let Err(e) = logger::init(
        logger::Level::Debug,
        logger::Target::Both,
        Some("logs/server.log"),
    ) {
        eprintln!("Logger init failed: {e}");
    }

    logger::info("════════════════════════════════════════");
    logger::info("  Mafia II: DE Multiplayer Server");
    logger::info(&format!("  Protocol v{}", protocol::PROTOCOL_VERSION));
    logger::info(&format!("  Max players: {}", protocol::MAX_PLAYERS));
    logger::info(&format!("  Port: {}", protocol::DEFAULT_PORT));
    logger::info("════════════════════════════════════════");

    // TODO: запуск сетевого цикла
    logger::info("Server is not yet implemented. Coming soon!");
}