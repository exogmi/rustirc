mod utils;
mod models;
mod commands;
mod server;

#[cfg(test)]
mod tests;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    server::listener::start_server("127.0.0.1:6667").await
}
