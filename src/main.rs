mod utils;
mod models;
mod commands;
mod server;

#[cfg(test)]
mod tests;

use clap::{App, Arg};
use env_logger;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();

    // Parse command-line arguments
    let matches = App::new("IRC Server")
        .version("1.0")
        .author("Your Name")
        .about("A simple IRC server")
        .arg(Arg::with_name("bind")
            .short("b")
            .long("bind")
            .value_name("IP")
            .help("Sets the IP address to bind to")
            .takes_value(true))
        .arg(Arg::with_name("port")
            .short("p")
            .long("port")
            .value_name("PORT")
            .help("Sets the port to listen on")
            .takes_value(true))
        .get_matches();

    // Set bind IP and port
    let bind_ip = matches.value_of("bind").unwrap_or("0.0.0.0");
    let port = matches.value_of("port").unwrap_or("6667");
    let bind_address = format!("{}:{}", bind_ip, port);

    log::info!("Starting IRC server on {}", bind_address);

    // Start the server
    server::listener::start_server(&bind_address).await
}
