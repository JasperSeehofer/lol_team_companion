//! The reset-password command: mint a one-time password reset code for an
//! account (M137 item 5; there is no mailer). It is the same code an admin
//! mints on `/admin/resets`, for when no admin can sign in.
//!
//! ```text
//! cargo run --features ssr --bin reset-password -- someone@example.com
//! /app/lol_team_companion reset-password someone@example.com   # the server binary (container)
//! ```
//!
//! The code is printed once (never logged) and works once, for 15 minutes
//! from now: start the server again, send the code to the player privately,
//! and the player sets a new password on `/auth/reset`. The password itself
//! does not change until then. Like `promote`, the command opens the
//! database directly (`SURREAL_DATA_DIR`, default `./data`): stop the server
//! first, SurrealKV allows one process per data directory (docs/RUNBOOK.md).

use std::env;

use crate::server::{db, password};

const USAGE: &str = "Usage: reset-password <email>";

/// Run the command with its arguments (without the program name) and
/// return the process exit code.
pub async fn run(args: &[String]) -> i32 {
    let email = match args {
        [email] if !email.starts_with('-') && email.contains('@') => email.clone(),
        _ => {
            eprintln!("{USAGE}");
            return 2;
        }
    };

    let data_dir = env::var("SURREAL_DATA_DIR").unwrap_or_else(|_| "./data".to_string());
    let conn = match db::init_db(&data_dir).await {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Could not open the database at {data_dir}: {e}");
            eprintln!(
                "If the server is running, stop it and wait for it to exit: SurrealKV allows \
                 one process per data directory."
            );
            return 1;
        }
    };

    match password::mint_reset_code(&conn, &email, None).await {
        Ok(m) => {
            println!(
                "Reset code for {} ({email}, {}):\n\n    {}\n\nIt works once, until {} (UTC, {} \
                 minutes from now). Start the server, send the code to the player in a private \
                 message, and they set a new password on /auth/reset with their email and the \
                 code. The code is not shown again.",
                m.username,
                m.user_id,
                m.code,
                m.expires_at,
                password::RESET_CODE_TTL_MINUTES
            );
            0
        }
        Err(db::DbError::NotFound) => {
            eprintln!("No account signs in with {email}.");
            1
        }
        Err(e) => {
            eprintln!("Creating the reset code failed: {e}");
            1
        }
    }
}
