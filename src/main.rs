mod command;
mod error;
mod server;

const DEFAULT_SECRET: &'static str = "secret";
const DEFAULT_CONN: &'static str = "host=localhost";

#[tokio::main]
async fn main() {
    server::serve().await;
}
