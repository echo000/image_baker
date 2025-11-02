/// Initialize application logging
pub fn init_logging() {
    use tracing::Level;
    use tracing::subscriber::set_global_default;
    use tracing_subscriber::FmtSubscriber;

    // all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
    // will be written to stdout.
    let _ = set_global_default(
        FmtSubscriber::builder()
            .with_max_level(Level::DEBUG)
            .finish(),
    );
}
