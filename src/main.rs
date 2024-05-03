use zero2prod::configuration::get_configuration;

// Procedural macro which initializes an async runtime that block on (drives) HttpServer::run
// returned futures to completion.
#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let configuration = get_configuration().expect("Failed to read configuration.");
    let listener =
        std::net::TcpListener::bind(("127.0.0.1", configuration.application_port)).expect(
            &format!("Failed to bind port {}", configuration.application_port),
        );
    // Bubble up the io::Error  if we failed to bind the address
    // Otherwise call .await on the Server
    zero2prod::startup::run(listener)?.await
}
