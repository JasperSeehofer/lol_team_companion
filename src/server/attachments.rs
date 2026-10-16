//! Screenshot attachments of feedback reports (M141 H2).
//!
//! One image per report, stored as `<dir>/<report key>.webp` under
//! `ATTACHMENTS_DIR` (a volume, never the database). Uploads are checked by
//! their first bytes (PNG, JPEG or WebP, whatever the browser claims),
//! capped at [`MAX_UPLOAD_BYTES`], decoded with size limits and re-encoded
//! to WebP, which drops every metadata block (EXIF, location, comments);
//! the stored file is at most [`MAX_STORED_BYTES`] (larger pictures are
//! scaled down). Nothing uploaded is ever executed or served as uploaded:
//! only the re-encoded WebP is served, to admins only, as `image/webp`.
//!
//! The directory is installed once at startup ([`init`]); without it (unit
//! and integration tests, tools) nothing is written, read or deleted, so a
//! test can never touch a live path. Pure helpers take the directory.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// Largest upload accepted, in bytes.
pub const MAX_UPLOAD_BYTES: usize = 5 * 1024 * 1024;
/// Largest stored WebP, in bytes.
pub const MAX_STORED_BYTES: usize = 1024 * 1024;
/// Largest width or height decoded.
pub const MAX_DIMENSION: u32 = 8192;
/// Smallest width a picture is scaled down to before giving up.
const MIN_WIDTH: u32 = 320;
/// How long after sending a report its screenshot may be attached.
pub const ATTACH_WINDOW_SECS: i64 = 15 * 60;

static DIR: OnceLock<PathBuf> = OnceLock::new();

/// Install the attachments directory (startup only; creates it).
pub fn init(dir: PathBuf) -> std::io::Result<()> {
    std::fs::create_dir_all(&dir)?;
    let _ = DIR.set(dir);
    Ok(())
}

/// The installed directory, if any.
pub fn dir() -> Option<&'static Path> {
    DIR.get().map(PathBuf::as_path)
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum AttachError {
    #[error("The image is too large (up to 5 MB).")]
    TooLarge,
    #[error("Attach a PNG, JPEG or WebP image.")]
    NotAnImage,
    #[error("The image could not be read.")]
    Unreadable,
    #[error("The image is too large to keep under 1 MB.")]
    TooDetailed,
}

/// The picture types accepted, told by their first bytes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Png,
    Jpeg,
    Webp,
}

/// What the bytes are, by magic number only.
pub fn sniff(bytes: &[u8]) -> Option<Kind> {
    if bytes.starts_with(&[0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A]) {
        Some(Kind::Png)
    } else if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
        Some(Kind::Jpeg)
    } else if bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        Some(Kind::Webp)
    } else {
        None
    }
}

/// Decode an upload and re-encode it as a lossless WebP of at most
/// [`MAX_STORED_BYTES`]. CPU-bound: call it from `spawn_blocking`.
pub fn to_webp(bytes: &[u8]) -> Result<Vec<u8>, AttachError> {
    use image::{ImageFormat, ImageReader, Limits};
    if bytes.len() > MAX_UPLOAD_BYTES {
        return Err(AttachError::TooLarge);
    }
    let format = match sniff(bytes).ok_or(AttachError::NotAnImage)? {
        Kind::Png => ImageFormat::Png,
        Kind::Jpeg => ImageFormat::Jpeg,
        Kind::Webp => ImageFormat::WebP,
    };
    let mut reader = ImageReader::with_format(std::io::Cursor::new(bytes), format);
    let mut limits = Limits::default();
    limits.max_image_width = Some(MAX_DIMENSION);
    limits.max_image_height = Some(MAX_DIMENSION);
    limits.max_alloc = Some(256 * 1024 * 1024);
    reader.limits(limits);
    let img = reader.decode().map_err(|_| AttachError::Unreadable)?;
    // Screenshots have no useful alpha; RGB keeps the file small.
    let mut rgb = img.to_rgb8();
    loop {
        let out = encode_webp(&rgb)?;
        if out.len() <= MAX_STORED_BYTES {
            return Ok(out);
        }
        let (w, h) = rgb.dimensions();
        if w <= MIN_WIDTH {
            return Err(AttachError::TooDetailed);
        }
        let nw = ((w as f32) * 0.75).round().max(MIN_WIDTH as f32) as u32;
        let nh = ((h as f32) * (nw as f32 / w as f32)).round().max(1.0) as u32;
        rgb = image::imageops::resize(&rgb, nw, nh, image::imageops::FilterType::Triangle);
    }
}

fn encode_webp(rgb: &image::RgbImage) -> Result<Vec<u8>, AttachError> {
    let mut out = Vec::new();
    image::codecs::webp::WebPEncoder::new_lossless(&mut out)
        .encode(
            rgb.as_raw(),
            rgb.width(),
            rgb.height(),
            image::ExtendedColorType::Rgb8,
        )
        .map_err(|_| AttachError::Unreadable)?;
    Ok(out)
}

/// The file name for a report (`bug_report:abc` or `abc`), or `None` when
/// the key is not a plain record key (never a path).
pub fn file_name(report_id: &str) -> Option<String> {
    let key = report_id.strip_prefix("bug_report:").unwrap_or(report_id);
    let key = key.trim_start_matches('⟨').trim_end_matches('⟩');
    let ok = !key.is_empty()
        && key.len() <= 64
        && key
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-');
    ok.then(|| format!("{key}.webp"))
}

/// The stored file of a report under `dir`.
pub fn path_in(dir: &Path, report_id: &str) -> Option<PathBuf> {
    file_name(report_id).map(|f| dir.join(f))
}

/// Whether a report has a stored screenshot.
pub fn exists_in(dir: &Path, report_id: &str) -> bool {
    path_in(dir, report_id).is_some_and(|p| p.is_file())
}

/// Write a report's screenshot (atomically: temp file, then rename).
pub fn store_in(dir: &Path, report_id: &str, webp: &[u8]) -> std::io::Result<PathBuf> {
    let path = path_in(dir, report_id)
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "bad report id"))?;
    std::fs::create_dir_all(dir)?;
    crate::server::bug_report_export::atomic_write(&path, webp)?;
    Ok(path)
}

/// Delete a report's screenshot; `Ok(false)` when there was none.
pub fn remove_in(dir: &Path, report_id: &str) -> std::io::Result<bool> {
    let Some(path) = path_in(dir, report_id) else {
        return Ok(false);
    };
    match std::fs::remove_file(path) {
        Ok(()) => Ok(true),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
        Err(e) => Err(e),
    }
}

/// [`exists_in`] in the installed directory (false without one).
pub fn exists(report_id: &str) -> bool {
    dir().is_some_and(|d| exists_in(d, report_id))
}

/// Delete the screenshots of these reports in the installed directory
/// (account deletion, report deletion). Errors are logged, not returned:
/// the rows are already gone or anonymised.
pub fn remove_all(report_ids: &[String]) -> usize {
    let Some(d) = dir() else { return 0 };
    let mut n = 0;
    for id in report_ids {
        match remove_in(d, id) {
            Ok(true) => n += 1,
            Ok(false) => {}
            Err(e) => tracing::error!(report = %id, "could not delete screenshot: {e}"),
        }
    }
    n
}

/// Decoding is memory-hungry: one at a time.
static DECODE: tokio::sync::Semaphore = tokio::sync::Semaphore::const_new(1);

fn plain(status: axum::http::StatusCode, msg: &str) -> axum::response::Response {
    use axum::response::IntoResponse;
    (
        status,
        [
            (
                axum::http::header::CONTENT_TYPE,
                "text/plain; charset=utf-8",
            ),
            (axum::http::header::CACHE_CONTROL, "no-store"),
        ],
        msg.to_string(),
    )
        .into_response()
}

/// `POST /feedback/attachment/{report}`: the raw image bytes of the
/// screenshot for a report the signed-in user sent in the last
/// [`ATTACH_WINDOW_SECS`]. One per report: sending a report takes a
/// `Bucket::Feedback` token, so attachments are limited with it. The route
/// caps the body at [`MAX_UPLOAD_BYTES`]; the session cookie is
/// SameSite=Strict, so another site cannot post with it.
pub async fn upload_handler(
    auth: crate::server::auth::AuthSession,
    axum::extract::Path(report): axum::extract::Path<String>,
    body: axum::body::Bytes,
) -> axum::response::Response {
    use axum::http::StatusCode;
    let Some(user) = auth.user else {
        return plain(StatusCode::UNAUTHORIZED, "Sign in to attach a screenshot.");
    };
    let (Some(dir), Some(db)) = (dir(), crate::server::db::shared()) else {
        return plain(
            StatusCode::SERVICE_UNAVAILABLE,
            "Screenshots cannot be stored right now.",
        );
    };
    if file_name(&report).is_none() {
        return plain(StatusCode::NOT_FOUND, "Not found");
    }
    match crate::server::community::store::report_open_for_attachment(&db, &user.id, &report).await
    {
        Ok(()) => {}
        Err(crate::server::db::DbError::NotFound) => {
            return plain(
                StatusCode::NOT_FOUND,
                "This report can no longer take a screenshot.",
            )
        }
        Err(e) => {
            tracing::error!(user = %user.id, "attachment check failed: {e}");
            return plain(
                StatusCode::INTERNAL_SERVER_ERROR,
                "The screenshot was not saved.",
            );
        }
    }
    if exists_in(dir, &report) {
        return plain(
            StatusCode::CONFLICT,
            "This report already has a screenshot.",
        );
    }
    if body.len() > MAX_UPLOAD_BYTES {
        return plain(
            StatusCode::PAYLOAD_TOO_LARGE,
            &AttachError::TooLarge.to_string(),
        );
    }
    let Ok(_permit) = DECODE.acquire().await else {
        return plain(
            StatusCode::SERVICE_UNAVAILABLE,
            "Screenshots cannot be stored right now.",
        );
    };
    let webp = match tokio::task::spawn_blocking(move || to_webp(&body)).await {
        Ok(Ok(w)) => w,
        Ok(Err(e)) => return plain(StatusCode::UNPROCESSABLE_ENTITY, &e.to_string()),
        Err(e) => {
            tracing::error!("screenshot decode task failed: {e}");
            return plain(
                StatusCode::INTERNAL_SERVER_ERROR,
                "The screenshot was not saved.",
            );
        }
    };
    if let Err(e) = store_in(dir, &report, &webp) {
        tracing::error!(report = %report, "could not store screenshot: {e}");
        return plain(
            StatusCode::INTERNAL_SERVER_ERROR,
            "The screenshot was not saved.",
        );
    }
    tracing::info!(user = %user.id, report = %report, bytes = webp.len(), "screenshot attached");
    crate::server::bug_report_export::request_export();
    plain(StatusCode::CREATED, "Saved")
}

/// `GET /admin/attachment/{report}`: the stored WebP, for admins only
/// (everyone else gets 404, as on every admin surface). Served as
/// `image/webp` with `nosniff`, no caching and a CSP that allows nothing.
pub async fn serve_handler(
    auth: crate::server::auth::AuthSession,
    axum::extract::Path(report): axum::extract::Path<String>,
) -> axum::response::Response {
    use axum::http::{header, StatusCode};
    use axum::response::IntoResponse;
    let admin = auth.user.as_ref().is_some_and(|u| u.is_admin);
    let path = dir().and_then(|d| path_in(d, &report));
    let (true, Some(path)) = (admin, path) else {
        return plain(StatusCode::NOT_FOUND, "Not found");
    };
    match tokio::fs::read(&path).await {
        Ok(bytes) if sniff(&bytes) == Some(Kind::Webp) => (
            StatusCode::OK,
            [
                (header::CONTENT_TYPE, "image/webp"),
                (header::X_CONTENT_TYPE_OPTIONS, "nosniff"),
                (header::CACHE_CONTROL, "private, no-store"),
                (
                    header::CONTENT_SECURITY_POLICY,
                    "default-src 'none'; sandbox",
                ),
                (header::CONTENT_DISPOSITION, "inline"),
            ],
            bytes,
        )
            .into_response(),
        _ => plain(StatusCode::NOT_FOUND, "Not found"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!(
            "companion-attach-{tag}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        ));
        std::fs::create_dir_all(&d).expect("temp dir");
        d
    }

    fn png(w: u32, h: u32, noisy: bool) -> Vec<u8> {
        let mut seed: u32 = 12345;
        let img = image::RgbImage::from_fn(w, h, |x, y| {
            if noisy {
                seed = seed.wrapping_mul(1_103_515_245).wrapping_add(12345);
                let v = (seed >> 8) as u8;
                image::Rgb([v, v.wrapping_mul(3), v.wrapping_add(x as u8)])
            } else {
                image::Rgb([(x % 256) as u8, (y % 256) as u8, 40])
            }
        });
        let mut out = Vec::new();
        image::DynamicImage::ImageRgb8(img)
            .write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png)
            .expect("png");
        out
    }

    #[test]
    fn sniffs_by_magic_bytes_only() {
        assert_eq!(sniff(&png(2, 2, false)), Some(Kind::Png));
        assert_eq!(sniff(&[0xFF, 0xD8, 0xFF, 0xE0]), Some(Kind::Jpeg));
        assert_eq!(sniff(b"RIFF\0\0\0\0WEBPVP8 "), Some(Kind::Webp));
        assert_eq!(sniff(b"<svg xmlns="), None);
        assert_eq!(sniff(b"GIF89a"), None);
        assert_eq!(sniff(b""), None);
    }

    #[test]
    fn re_encodes_to_webp_and_refuses_the_rest() {
        let out = to_webp(&png(64, 48, false)).expect("webp");
        assert_eq!(sniff(&out), Some(Kind::Webp));
        assert!(out.len() <= MAX_STORED_BYTES);
        let back = image::load_from_memory_with_format(&out, image::ImageFormat::WebP).unwrap();
        assert_eq!((back.width(), back.height()), (64, 48));
        assert_eq!(
            to_webp(b"<script>alert(1)</script>"),
            Err(AttachError::NotAnImage)
        );
        // Magic bytes of a PNG with garbage after them.
        let mut fake = png(2, 2, false)[..8].to_vec();
        fake.extend_from_slice(b"not really");
        assert_eq!(to_webp(&fake), Err(AttachError::Unreadable));
        assert_eq!(
            to_webp(&vec![0u8; MAX_UPLOAD_BYTES + 1]),
            Err(AttachError::TooLarge)
        );
    }

    #[test]
    fn metadata_does_not_survive() {
        // A PNG with a text chunk (tEXt "Comment") before IEND.
        let mut bytes = png(8, 8, false);
        let iend = bytes.len() - 12;
        let data = b"Comment\0secret-location";
        let mut chunk = (data.len() as u32).to_be_bytes().to_vec();
        chunk.extend_from_slice(b"tEXt");
        chunk.extend_from_slice(data);
        let crc_input: Vec<u8> = b"tEXt".iter().chain(data.iter()).copied().collect();
        chunk.extend_from_slice(&crc32(&crc_input).to_be_bytes());
        bytes.splice(iend..iend, chunk);
        assert!(bytes.windows(15).any(|w| w == b"secret-location"));
        let out = to_webp(&bytes).expect("webp");
        assert!(!out.windows(15).any(|w| w == b"secret-location"));
    }

    fn crc32(data: &[u8]) -> u32 {
        let mut crc = 0xFFFF_FFFFu32;
        for &b in data {
            crc ^= b as u32;
            for _ in 0..8 {
                crc = if crc & 1 != 0 {
                    (crc >> 1) ^ 0xEDB8_8320
                } else {
                    crc >> 1
                };
            }
        }
        !crc
    }

    #[test]
    fn large_pictures_are_scaled_under_the_cap() {
        let out = to_webp(&png(1400, 900, true)).expect("webp");
        assert!(out.len() <= MAX_STORED_BYTES, "{} bytes", out.len());
    }

    #[test]
    fn file_names_are_plain_keys_only() {
        assert_eq!(
            file_name("bug_report:abc123").as_deref(),
            Some("abc123.webp")
        );
        assert_eq!(file_name("abc_1-2").as_deref(), Some("abc_1-2.webp"));
        assert_eq!(file_name("bug_report:⟨x9⟩").as_deref(), Some("x9.webp"));
        for bad in ["", "../etc/passwd", "a/b", "a.b", "bug_report:", "x y"] {
            assert_eq!(file_name(bad), None, "{bad}");
        }
    }

    #[test]
    fn store_exists_remove_in_a_temp_dir() {
        let d = temp_dir("store");
        assert!(!exists_in(&d, "bug_report:r1"));
        let out = to_webp(&png(4, 4, false)).unwrap();
        let p = store_in(&d, "bug_report:r1", &out).unwrap();
        assert!(p.starts_with(&d));
        assert!(exists_in(&d, "bug_report:r1"));
        assert!(remove_in(&d, "bug_report:r1").unwrap());
        assert!(!remove_in(&d, "bug_report:r1").unwrap());
        assert!(store_in(&d, "../x", &out).is_err());
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn nothing_happens_without_an_installed_dir() {
        // Unit tests never call `init`.
        assert!(dir().is_none());
        assert!(!exists("bug_report:r1"));
        assert_eq!(remove_all(&["bug_report:r1".into()]), 0);
    }
}
