// Procedural macro which initializes an async runtime that block on (drives) HttpServer::run
// returned futures to completion.
#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    let listener =
        std::net::TcpListener::bind(("127.0.0.1", 8000)).expect("Failed to bind pot 8000.");
    // Bubble up the io::Error  if we failed to bind the address
    // Otherwise call .await on the Server
    zero2prod::run(listener)?.await
}
