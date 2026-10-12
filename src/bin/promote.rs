//! The promote command (see `server::promote`): make an existing account an
//! admin, or take that back. Stop the server first.
//!
//!     cargo run --features ssr --bin promote -- someone@example.com
//!     cargo run --features ssr --bin promote -- --revoke someone@example.com
//!     cargo run --features ssr --bin promote -- --list

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let args: Vec<String> = std::env::args().skip(1).collect();
    std::process::exit(lol_team_companion::server::promote::run(&args).await);
}
