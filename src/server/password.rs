//! Password change and admin password reset (M137 item 5). There is no
//! mailer, so a forgotten password is reset through an admin:
//!
//! - **Change** (signed in, `/profile`): the current password is verified,
//!   the new one hashed with the same Argon2 parameters as registration
//!   ([`set_password_hash`]); the caller ends every session of the account
//!   and signs the current one in again.
//! - **Reset code** ([`mint_reset_code`]): an admin on `/admin/resets`, or the
//!   `reset-password <email>` command with the server stopped, creates a
//!   one-time code for an account. The code is 16 characters from a
//!   32-letter alphabet (80 bits from the OS random source), shown once and
//!   stored only as its SHA-256 hash ([`hash_reset_code`]). It works once and
//!   for [`RESET_CODE_TTL_MINUTES`] minutes; minting a new one for the same
//!   account deletes the account's codes that were not used yet.
//! - **Redeem** ([`redeem_reset_code`], public `/auth/reset`): email, code and
//!   new password. The open codes of that account are compared with the
//!   hash of the given code in constant time ([`hashes_match`]); the match
//!   is used and the password set in one transaction that re-checks "not
//!   used, not expired" in the database, under the invariant lock, so two
//!   concurrent redeems cannot both succeed. Every failure (unknown email,
//!   wrong, used or expired code) is the same [`DbError::NotFound`]. The
//!   caller then ends every session of the account.
//!
//! A plain hash (not an HMAC keyed with `SESSION_SECRET`) because the
//! command runs without the server's configuration; with 80 random bits a
//! leaked hash cannot be inverted by trying codes.

use serde::Deserialize;
use surrealdb::{
    engine::local::Db,
    types::{Datetime, RecordId, SurrealValue, ToSql},
    Surreal,
};

use crate::server::db::{
    check_tx, dt_to_string, invariant_lock, normalize_email, record_key, thrown_tag, DbError,
    DbResult,
};

/// How long a reset code works.
pub const RESET_CODE_TTL_MINUTES: i64 = 15;

/// Characters of a reset code: no 0/O, 1/I. 32 letters, so a random byte
/// modulo 32 is unbiased.
const ALPHABET: &[u8] = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
/// Characters in a code (80 bits).
const CODE_LEN: usize = 16;

/// Minimum password length (the same rule as registration).
pub const MIN_PASSWORD_LEN: usize = crate::pages::auth::register::MIN_PASSWORD_LEN;
/// Longest password accepted, in bytes (the same rule as sign-in).
pub const MAX_PASSWORD_BYTES: usize = 1024;

/// The rule for a new password, shared by change and reset. The message is
/// safe to show.
pub fn validate_new_password(password: &str) -> Result<(), String> {
    if password.chars().count() < MIN_PASSWORD_LEN {
        return Err(format!(
            "Password must be at least {MIN_PASSWORD_LEN} characters."
        ));
    }
    if password.len() > MAX_PASSWORD_BYTES {
        return Err("Password is too long.".to_string());
    }
    Ok(())
}

/// A new random reset code such as `K7QM-2XWD-9HPA-R4TB`.
pub fn generate_reset_code() -> String {
    use password_hash::rand_core::{OsRng, RngCore};
    let mut bytes = [0u8; CODE_LEN];
    OsRng.fill_bytes(&mut bytes);
    let chars: Vec<char> = bytes
        .iter()
        .map(|b| ALPHABET[(*b as usize) % ALPHABET.len()] as char)
        .collect();
    chars
        .chunks(4)
        .map(|c| c.iter().collect::<String>())
        .collect::<Vec<_>>()
        .join("-")
}

/// A code as typed or pasted: uppercase, without spaces and dashes.
pub fn normalize_reset_code(code: &str) -> String {
    code.chars()
        .filter(|c| !c.is_whitespace() && *c != '-')
        .flat_map(char::to_uppercase)
        .collect()
}

/// What is stored for a code: lowercase hex SHA-256 of its normalised form.
pub fn hash_reset_code(code: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut h = Sha256::new();
    h.update(b"companion-reset-v1:");
    h.update(normalize_reset_code(code).as_bytes());
    h.finalize().iter().map(|b| format!("{b:02x}")).collect()
}

/// Constant-time equality of two stored hashes.
pub fn hashes_match(a: &str, b: &str) -> bool {
    use subtle::ConstantTimeEq;
    a.len() == b.len() && bool::from(a.as_bytes().ct_eq(b.as_bytes()))
}

/// A freshly minted code: shown once, never stored in clear.
#[derive(Clone, Debug)]
pub struct MintedReset {
    pub code: String,
    pub user_id: String,
    pub username: String,
    /// RFC 3339, UTC.
    pub expires_at: String,
}

/// Set the stored password hash of a user. Every session of the user stops
/// matching (the session auth hash is derived from it).
pub async fn set_password_hash(db: &Surreal<Db>, user_id: &str, hash: String) -> DbResult<()> {
    let mut r = db
        .query("UPDATE type::record('user', $k) SET password_hash = $hash RETURN VALUE id")
        .bind(("k", record_key(user_id, "user")))
        .bind(("hash", hash))
        .await?
        .check()?;
    let ids: Vec<RecordId> = r.take(0)?;
    if ids.is_empty() {
        return Err(DbError::NotFound);
    }
    Ok(())
}

/// Mint a reset code for the account that signs in with `email`.
/// `created_by` is the admin (`None` for the command); a creator who is not a
/// stored admin is refused like an unknown account (the server function
/// checks `require_admin` first; this is the second lock). Earlier unused
/// codes of the account are deleted. `DbError::NotFound` for an unknown
/// email (and for the tombstone account).
pub async fn mint_reset_code(
    db: &Surreal<Db>,
    email: &str,
    created_by: Option<&str>,
) -> DbResult<MintedReset> {
    #[derive(Debug, Deserialize, SurrealValue)]
    struct Row {
        user: RecordId,
        username: String,
        expires_at: Datetime,
    }
    let code = generate_reset_code();
    let r = db
        .query(
            "BEGIN TRANSACTION; \
             LET $u = (SELECT VALUE id FROM user WHERE email = $email AND id != user:former_member LIMIT 1)[0]; \
             IF $u = NONE { THROW 'NO_USER' }; \
             LET $by = IF $by_key = NONE { NONE } ELSE { type::record('user', $by_key) }; \
             IF $by != NONE AND array::len(SELECT VALUE id FROM user WHERE id = $by AND (is_admin = true OR admin_promoted_at != NONE)) = 0 { THROW 'NOT_ADMIN' }; \
             DELETE password_reset WHERE user = $u AND used_at = NONE; \
             LET $row = (CREATE ONLY password_reset SET user = $u, code_hash = $hash, created_by = $by, \
                 expires_at = time::now() + <duration>$ttl); \
             RETURN { user: $u, username: $u.username, expires_at: $row.expires_at }; \
             COMMIT TRANSACTION;",
        )
        .bind(("email", normalize_email(email)))
        .bind(("by_key", created_by.map(|id| record_key(id, "user"))))
        .bind(("hash", hash_reset_code(&code)))
        .bind(("ttl", format!("{RESET_CODE_TTL_MINUTES}m")))
        .await?;
    let mut r = match check_tx(r) {
        Ok(r) => r,
        Err(e) => {
            return Err(match thrown_tag(&e).as_deref() {
                Some("NO_USER") | Some("NOT_ADMIN") => DbError::NotFound,
                _ => e,
            })
        }
    };
    let row: Option<Row> = r.take(7)?;
    let row = row.ok_or_else(|| DbError::Other("reset code not created".into()))?;
    Ok(MintedReset {
        code,
        user_id: row.user.to_sql(),
        username: row.username,
        expires_at: dt_to_string(&row.expires_at),
    })
}

/// Use the reset code `code` of the account that signs in with `email` and
/// set its password hash to `new_hash` (hashed by the caller, before this
/// call, so a wrong code costs the same time as a right one). Returns the
/// user id. Every failure is `DbError::NotFound`.
pub async fn redeem_reset_code(
    db: &Surreal<Db>,
    email: &str,
    code: &str,
    new_hash: String,
) -> DbResult<String> {
    #[derive(Debug, Deserialize, SurrealValue)]
    struct Open {
        id: RecordId,
        user: RecordId,
        code_hash: String,
    }
    let given = hash_reset_code(code);
    let _guard = invariant_lock().await;
    let mut r = db
        .query(
            "SELECT id, user, code_hash FROM password_reset \
             WHERE user = (SELECT VALUE id FROM user WHERE email = $email AND id != user:former_member LIMIT 1)[0] \
             AND used_at = NONE AND expires_at > time::now()",
        )
        .bind(("email", normalize_email(email)))
        .await?;
    let open: Vec<Open> = r.take(0).unwrap_or_default();
    // Compare with every open code, without stopping at the first match.
    let mut hit: Option<&Open> = None;
    for row in &open {
        if hashes_match(&row.code_hash, &given) {
            hit = Some(row);
        }
    }
    let Some(hit) = hit else {
        return Err(DbError::NotFound);
    };
    let r = db
        .query(
            "BEGIN TRANSACTION; \
             LET $used = (UPDATE $reset SET used_at = time::now() \
                 WHERE user = $u AND used_at = NONE AND expires_at > time::now() RETURN VALUE id); \
             IF array::len($used) = 0 { THROW 'GONE' }; \
             UPDATE $u SET password_hash = $hash; \
             COMMIT TRANSACTION;",
        )
        .bind(("reset", hit.id.clone()))
        .bind(("u", hit.user.clone()))
        .bind(("hash", new_hash))
        .await?;
    match check_tx(r) {
        Ok(_) => Ok(hit.user.to_sql()),
        Err(e) if thrown_tag(&e).as_deref() == Some("GONE") => Err(DbError::NotFound),
        Err(e) => Err(e),
    }
}

/// The newest reset codes for the admin page: who for, who made them, when,
/// and whether they were used (never the hash).
pub async fn list_recent_resets(
    db: &Surreal<Db>,
    limit: i64,
) -> DbResult<Vec<crate::models::password_reset::ResetSummary>> {
    #[derive(Debug, Deserialize, SurrealValue)]
    struct Row {
        username: Option<String>,
        created_by_name: Option<String>,
        created_at: Datetime,
        expires_at: Datetime,
        used_at: Option<Datetime>,
    }
    let mut r = db
        .query(
            "SELECT user.username AS username, created_by.username AS created_by_name, \
             created_at, expires_at, used_at FROM password_reset \
             ORDER BY created_at DESC LIMIT $limit",
        )
        .bind(("limit", limit))
        .await?;
    let rows: Vec<Row> = r.take(0)?;
    Ok(rows
        .into_iter()
        .map(|r| crate::models::password_reset::ResetSummary {
            username: r.username.unwrap_or_default(),
            created_by: r.created_by_name,
            created_at: dt_to_string(&r.created_at),
            expires_at: dt_to_string(&r.expires_at),
            used_at: crate::server::db::opt_dt(r.used_at),
        })
        .collect())
}

/// Delete reset rows that expired more than 30 days ago (the hourly purge).
pub async fn purge_old_resets(db: &Surreal<Db>) -> DbResult<()> {
    db.query("DELETE password_reset WHERE expires_at < time::now() - 30d")
        .await?
        .check()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn codes_are_long_random_and_readable() {
        let a = generate_reset_code();
        let b = generate_reset_code();
        assert_ne!(a, b);
        assert_eq!(a.len(), 19, "{a}");
        assert_eq!(normalize_reset_code(&a).len(), CODE_LEN);
        assert!(normalize_reset_code(&a)
            .bytes()
            .all(|c| ALPHABET.contains(&c)));
        // 16 characters of a 32-letter alphabet: 80 bits.
        assert_eq!(ALPHABET.len(), 32);
        const { assert!(CODE_LEN * 5 >= 80) };
    }

    #[test]
    fn the_hash_ignores_case_spaces_and_dashes() {
        let h = hash_reset_code("K7QM-2XWD-9HPA-R4TB");
        assert_eq!(h.len(), 64);
        assert!(h.bytes().all(|c| c.is_ascii_hexdigit()));
        assert_eq!(h, hash_reset_code(" k7qm 2xwd9hpa-r4tb "));
        assert_ne!(h, hash_reset_code("K7QM-2XWD-9HPA-R4TC"));
        assert_ne!(h, "K7QM2XWD9HPAR4TB", "the code itself is never stored");
    }

    #[test]
    fn hashes_compare_exactly() {
        let h = hash_reset_code("AAAA-BBBB-CCCC-DDDD");
        assert!(hashes_match(&h, &h.clone()));
        assert!(!hashes_match(&h, &hash_reset_code("AAAA-BBBB-CCCC-DDDE")));
        assert!(!hashes_match(&h, &h[..63]));
        assert!(!hashes_match(&h, ""));
    }

    /// The queries exclude the tombstone account by its literal id.
    #[test]
    fn the_tombstone_is_excluded_by_its_id() {
        assert_eq!(
            crate::server::account::FORMER_MEMBER_ID,
            "user:former_member"
        );
    }

    #[test]
    fn new_passwords_follow_the_registration_rule() {
        assert!(validate_new_password("short").is_err());
        assert!(validate_new_password("123456789").is_err());
        assert!(validate_new_password("1234567890").is_ok());
        assert!(validate_new_password(&"x".repeat(1025)).is_err());
        assert_eq!(MIN_PASSWORD_LEN, 10);
    }
}
