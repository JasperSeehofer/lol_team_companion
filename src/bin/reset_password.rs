//! The reset-password command (see `server::reset_password`): print a
//! one-time password reset code for an account. Stop the server first.
//!
//!     cargo run --features ssr --bin reset-password -- someone@example.com

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    let args: Vec<String> = std::env::args().skip(1).collect();
    std::process::exit(lol_team_companion::server::reset_password::run(&args).await);
}
