//! The promote command: make an existing account an admin, or take that back
//! (owner decision for Release 1: there is no email provider, so admins are
//! not invited by mail).
//!
//!     cargo run --features ssr --bin promote -- someone@example.com
//!     cargo run --features ssr --bin promote -- --revoke someone@example.com
//!     cargo run --features ssr --bin promote -- --list
//!     /app/lol_team_companion promote someone@example.com   # the server binary (container)
//!
//! A promoted account is an admin whether or not `ADMIN_EMAILS` lists it,
//! until it is revoked here. `ADMIN_EMAILS` keeps working as before. The
//! command opens the database directly (`SURREAL_DATA_DIR`, default
//! `./data`): stop the server first, SurrealKV allows one process per data
//! directory. In the container: `docker compose stop app`, then run the
//! binary in a one-off container on the same volume (docs/RUNBOOK.md).

use std::env;

use crate::server::db;

const USAGE: &str = "Usage: promote <email> | promote --revoke <email> | promote --list";

/// Run the command with its arguments (without the program name) and
/// return the process exit code.
pub async fn run(args: &[String]) -> i32 {
    let (action, email) = match args {
        [flag] if flag == "--list" => ("list", None),
        [flag, email] if flag == "--revoke" => ("revoke", Some(email.clone())),
        [email] if !email.starts_with('-') => ("promote", Some(email.clone())),
        _ => {
            eprintln!("{USAGE}");
            return 2;
        }
    };
    if let Some(e) = &email {
        if !e.contains('@') {
            eprintln!("{e:?} is not an email address.\n{USAGE}");
            return 2;
        }
    }

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

    match (action, email) {
        ("list", _) => match db::list_admins(&conn).await {
            Ok(rows) if rows.is_empty() => println!("No stored admins."),
            Ok(rows) => {
                for (email, username, promoted) in rows {
                    let how = if promoted {
                        "promoted with this command"
                    } else {
                        "through ADMIN_EMAILS (admin only while listed)"
                    };
                    println!("{email}  ({username}, {how})");
                }
            }
            Err(e) => return fail(&format!("Listing admins failed: {e}")),
        },
        (action, Some(email)) => {
            let promote = action == "promote";
            match db::set_admin_promotion(&conn, &email, promote).await {
                Ok((id, username)) if promote => println!(
                    "{username} ({email}, {id}) is now an admin. It takes effect on the next \
                     request; no sign-in needed."
                ),
                Ok((id, username)) => println!(
                    "{username} ({email}, {id}) is no longer an admin through this command. If \
                     ADMIN_EMAILS still lists the address, remove it there too."
                ),
                Err(db::DbError::NotFound) => {
                    return fail(&format!(
                    "No account signs in with {email}. Register it first (with an invite code), \
                     then promote it."
                    ))
                }
                Err(e) => return fail(&format!("Changing the admin status failed: {e}")),
            }
        }
        _ => return fail(USAGE),
    }
    0
}

fn fail(msg: &str) -> i32 {
    eprintln!("{msg}");
    1
}
