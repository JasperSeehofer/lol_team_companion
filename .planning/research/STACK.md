# Stack Research — v1.4 Immersive Companion: Room Rollout + AI Art Pipeline

**Domain:** Immersive 2.5D parallax rooms UI + layer-separated AI environment art pipeline
**Researched:** 2026-06-03
**Confidence:** HIGH (core findings verified via official docs/APIs; pricing details MEDIUM)

---

## Verdict: No Rust/Cargo dependency changes required

The proven Solo Journey room system rolls out to War Room, Hall of Records, and Throne using
zero new Rust crates. All needed web-sys types are already declared or pulled in transitively.
The only additions are **external tooling** (CLI tools for the art pipeline) and a **new
environment variable** (`BFL_KEY`). The Cargo.toml is untouched.

---

## Recommended Stack (additions only — existing stack unchanged)

### Core Technologies (unchanged)

The full Rust nightly / Leptos 0.8 / Axum 0.8 / SurrealDB 3.x / Tailwind CSS v4 stack is
already validated. See PROJECT.md. No changes.

---

### AI Image Generation

| Technology | Version / Endpoint | Purpose | Why Recommended |
|---|---|---|---|
| **Black Forest Labs FLUX API** | FLUX.2 [pro] via `api.eu.bfl.ai` | Generate Arcane-style environment art layers | German company (Freiburg), explicit EU endpoint (`api.eu.bfl.ai`), GDPR-compliant, ~$0.045/image at 1920x1080. The model behind every locked FLUX prompt in `AI-IMAGES.md`. |
| **fal.ai** (fallback runner) | `fal-ai/flux-pro` | Alternative FLUX runner if BFL direct has availability issues | Already referenced in `AI-IMAGES.md`; faster cold-start than BFL direct; ~$0.05/image. US-hosted — prefer BFL for primary generation (see EU note). |

**EU note on fal.ai:** fal.ai is US-domiciled and processes data on US servers per its privacy
policy. For a project that values EU-tool preference, **prefer the BFL direct EU endpoint**
(`api.eu.bfl.ai`) for all primary generation. fal.ai is acceptable as a development fallback
because the parallax plane images are not personal data. Enterprise EU-residency on fal.ai
requires a custom arrangement — not worth pursuing for this use case.

**FLUX model selection:** Use `flux-2-pro` (FLUX.2 [pro]). It supersedes the older `flux-pro`
(FLUX.1 [pro]) referenced in `AI-IMAGES.md` with improved quality at the same price point.
The `AI-IMAGES.md` prompts are model-agnostic and transfer directly. `flux-dev` saves ~50% on
cost but is licensed for non-commercial use only — do not use for production assets.

**BFL API call pattern** (async polling, EU endpoint):
```bash
# 1. Submit generation
RESPONSE=$(curl -s -X POST https://api.eu.bfl.ai/v1/flux-2-pro \
  -H "x-key: $BFL_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "prompt": "...",
    "width": 1920, "height": 1080,
    "steps": 28, "seed": 7421337
  }')
POLLING_URL=$(echo $RESPONSE | jq -r '.polling_url')

# 2. Poll until Ready (use the returned polling_url, not a hardcoded endpoint)
until [ "$(curl -s -H "x-key: $BFL_KEY" "$POLLING_URL" | jq -r '.status')" = "Ready" ]; do
  sleep 2
done

# 3. Download result
curl -s -H "x-key: $BFL_KEY" "$POLLING_URL" | jq -r '.result.sample' | xargs curl -L -o out.png
```

---

### Background Removal / Layer Separation

| Technology | Endpoint | Purpose | Why Recommended |
|---|---|---|---|
| **fal.ai BiRefNet v2** | `fal-ai/birefnet/v2` | Remove background from mid/near/overlay planes to produce transparent PNG | State-of-art dual-pathway segmentation; returns PNG with alpha channel; handles complex painterly edges better than rembg/U2Net; available as fal.ai API, no local GPU needed. |
| **ImageMagick** (existing) | `magick` CLI | Post-process: crop, level, convert to WebP | Already installed and used; handles the PNG → WebP step after BiRefNet output. |

**Layer separation workflow** for mid/near/overlay planes:

1. Generate each plane as a full-frame image from FLUX (sky plane: opaque; mid/near/overlays:
   prompt only the depth band of interest — see locked prompts in `AI-IMAGES.md`).
2. Run the non-sky outputs through `fal-ai/birefnet/v2` to cut the subject from the generated
   background, producing a transparent PNG. Use `model: "heavy"` for complex edges.
3. Verify alpha mask quality visually. Re-run with `model: "heavy"` variant if edges are muddy.
4. Convert to WebP: `magick in.png -quality 80 -define webp:alpha-quality=100 out.webp`.
5. Check per-region total stays under 400 KB budget.

**fal.ai BiRefNet call pattern:**
```bash
RESULT=$(curl -s -X POST https://fal.run/fal-ai/birefnet/v2 \
  -H "Authorization: Key $FAL_KEY" \
  -H "Content-Type: application/json" \
  -d '{"image_url": "https://...", "model": "heavy"}')
echo $RESULT | jq -r '.image.url' | xargs curl -L -o plane-mid-transparent.png
```

**Why not depth-map plane cutting?** Monocular depth estimation (MiDaS, ZoeDepth) generates a
continuous depth field, not clean layer masks. For stylized painterly art the sharp alpha cut
from BiRefNet composites more cleanly than a soft depth gradient. The prompt already encodes
depth intent (sky / midground / foreground), so the segmentation step just removes the
generated background — it does not need to reconstruct 3D.

**Why not rembg locally?** It requires adding a Python dependency chain to a pure Rust project.
fal.ai BiRefNet calls over HTTP from a shell script (same pattern as generation) with no local
installation. For ~36 total planes (6 planes × 3 rooms × 2 regions) the API cost is negligible.

---

### Image Optimization / Encoding

| Tool | When to Use | Why |
|---|---|---|
| **ImageMagick** (`magick`) | All encoding: PNG → WebP, PNG → JPEG | Already present; supports lossless and lossy WebP with alpha channel via `-define webp:alpha-quality=100`; sufficient quality for this use case. |
| **cwebp** (standalone) | NOT needed | Marginally better compression than ImageMagick's WebP encoder (<5% at q80) — not meaningful given the 400 KB budget has ample headroom (current 12 placeholders total ~52 KB). Adding a second encoder adds setup steps with no measurable benefit. |
| **avifenc** / AVIF format | NOT needed for this milestone | AVIF is 20-30% smaller than WebP at equivalent quality — useful for large photo sets. But: (a) AVIF encodes ~47x slower than WebP, (b) the `<Plane>` component uses `<img loading="lazy">` which benefits from fast WebP decoding in the compositor, (c) WebP already hits budget with headroom. Revisit only if per-room budget tightens below ~100 KB. |

**WebP encoding command for transparent planes/overlays** (lossless alpha, lossy RGB):
```bash
magick in.png -quality 80 -define webp:alpha-quality=100 out.webp
```

**WebP encoding command for sky plane** (opaque, fully lossy):
```bash
magick in.png -quality 80 out.webp
```

---

### Rust / WASM — No New Dependencies

**Verdict: Cargo.toml is unchanged.** All web-sys types used by the room system are already
declared or pulled in transitively via Leptos.

**Current web-sys features in Cargo.toml** (already sufficient for all three new rooms):
```
Window, Document, HtmlDocument, Element, HtmlElement, Node,
Storage, Navigator, Clipboard, SvgsvgElement, SvgGraphicsElement,
DomRect, MediaQueryList, CssStyleDeclaration,
DragEvent, DataTransfer
```

**Types used by room components and their coverage:**

| Type | Used In | Coverage |
|---|---|---|
| `web_sys::Window` | parallax.rs — RAF, matchMedia, innerWidth/Height, addEventListener/removeEventListener | Declared: `Window` feature covers all these methods. |
| `web_sys::HtmlElement` | parallax.rs — `style.set_property` for `--px`/`--py` | Declared: `HtmlElement`. |
| `web_sys::Element` | parallax.rs — `get_element_by_id` return type | Declared: `Element`. |
| `web_sys::CssStyleDeclaration` | parallax.rs — `set_property("--px", ...)` | Declared: `CssStyleDeclaration`. |
| `web_sys::MediaQueryList` | parallax.rs — prefers-reduced-motion gate | Declared: `MediaQueryList`. |
| `web_sys::MouseEvent` | parallax.rs — mousemove handler | Transitively provided by Leptos event system; WASM build succeeds without explicit declaration (verified via `cargo check --features hydrate --target wasm32-unknown-unknown`). |
| `web_sys::KeyboardEvent` | quick_nav.rs — Cmd+K / Escape handler | Transitively provided by Leptos; same verification. |

**No additional web-sys features are needed for the three remaining rooms.** War Room, Hall of
Records, and Throne use the same `AtmosphereRoot` / `Plane` / `GrowthOverlay` primitives as
Solo Journey. If a future room adds a novel browser API (e.g. `IntersectionObserver` for
lazy-loading, `TouchEvent` for mobile), add the corresponding feature at that point.

---

### JavaScript / Animation — No New Dependencies

**Verdict: No JS additions.** The parallax engine is pure CSS + web-sys (single RAF-batched
mousemove → `--px`/`--py` CSS vars → GPU-composited transforms). This architecture extends
directly to all new rooms with no modification. The QuickNav Cmd+K palette is pure Leptos +
web-sys keyboard listener — no JS library needed.

**Explicitly rejected additions:**

| Rejected | Why | What We Use Instead |
|---|---|---|
| GSAP Club plugins | Project guardrail — commercial license; explicitly banned | Pure CSS transitions/transforms |
| GSAP core (free) | Team preference: no heavy JS deps; adds ~70 KB gzipped; the RAF+CSS approach is already GPU-optimal for this use case | web-sys RAF + CSS `--px`/`--py` custom properties |
| Framer Motion / Motion One | React-ecosystem or JS runtime weight | Pure CSS |
| Three.js / Pixi.js | Full 3D/WebGL stack; 2.5D parallax does not need a canvas renderer; >300 KB | CSS `transform: translate3d()` on `<img>` planes |
| WASM physics crates (rapier, etc.) | No physics simulation; parallax is purely cosmetic | N/A |

---

### New Environment Variables

| Variable | Purpose | Where to Add |
|---|---|---|
| `BFL_KEY` | Black Forest Labs API key for FLUX.2 [pro] generation via `api.eu.bfl.ai` | `.env` (never committed); add to `.env.example` with a placeholder |
| `FAL_KEY` | fal.ai API key (already documented in `AI-IMAGES.md`, for BiRefNet + fallback FLUX runner) | `.env` (already in `.env.example`) |

Both are used only in offline art-generation shell scripts — never loaded by the server binary.
No changes to `src/main.rs`, `Cargo.toml`, or any Rust source.

---

## Art Pipeline Scripts (shell — not Rust)

The art generation pipeline lives outside Cargo entirely. Recommended structure under `scripts/`:

```
scripts/
├── gen-room-art.sh        # POST to BFL API, poll, download raw PNG per layer
├── remove-bg.sh           # POST to fal-ai/birefnet/v2, download transparent PNG
├── encode-webp.sh         # magick PNG → WebP at target quality/alpha settings
└── verify-budget.sh       # sum per-region file sizes, fail if >400 KB
```

Each script is independently runnable so a single-plane regeneration (prompt tweak, seed
change) does not re-run the full pipeline. This mirrors the "one-line binary swap" principle
documented in `AI-IMAGES.md`.

After generating real FLUX art, update each asset block in `AI-IMAGES.md` per the existing
instructions: record the assigned seed integer, update compute path, update final size, remove
the `TODO: Regenerate` line.

---

## Alternatives Considered

| Recommended | Alternative | Why Not |
|---|---|---|
| BFL direct EU endpoint (`api.eu.bfl.ai`) | fal.ai FLUX runner as primary | fal.ai is US-domiciled; EU preference per project values. BFL is the model author and offers explicit EU endpoint. |
| FLUX.2 [pro] | FLUX.1 [pro] (as in current `AI-IMAGES.md`) | FLUX.2 supersedes FLUX.1 at the same price; locked prompts transfer directly with no changes. |
| FLUX.2 [pro] | `flux-dev` | Non-commercial license only; cannot ship in a closed-beta product. |
| fal.ai BiRefNet v2 | Local rembg (Python) | Adds Python dependency chain to a pure Rust project; API call is sufficient for ~36 total planes. |
| fal.ai BiRefNet v2 | Depth-map cutting (MiDaS/ZoeDepth) | Continuous depth fields composite poorly with stylized painterly art; BiRefNet alpha masks are cleaner when the prompt already encodes depth intent. |
| ImageMagick (existing) | cwebp standalone | <5% size difference at q80; cwebp adds a second installed tool with no meaningful benefit given budget headroom. |
| ImageMagick (existing) | avifenc / AVIF | 47x slower encode; WebP already within budget; revisit only if budget tightens significantly. |
| Pure CSS + web-sys RAF | GSAP / Three.js | Project guardrails (GSAP Club banned, no heavy JS deps); GPU-composited CSS transforms are already optimal for this use case. |
| Existing web-sys features | Add `MouseEvent` / `KeyboardEvent` explicitly | Not needed — transitively provided by Leptos; WASM build verified passing without explicit declaration. |

---

## Version Compatibility

| Component | Current Version | Notes |
|---|---|---|
| web-sys | 0.3 | All room APIs covered by features already in Cargo.toml. No additions needed. |
| wasm-bindgen | 0.2 | `Closure::once`, `JsCast` patterns in parallax.rs stable with current version. |
| FLUX.2 [pro] (BFL API) | Production | EU endpoint: `https://api.eu.bfl.ai/v1/flux-2-pro`. Requires `x-key` header, async polling via returned `polling_url`. |
| fal.ai BiRefNet | v2 (`fal-ai/birefnet/v2`) | Returns PNG with alpha. Input: JPEG/PNG/WebP/AVIF up to 2048x2048. |
| ImageMagick | 6.9+ (system-installed) | WebP lossless alpha: `-quality 80 -define webp:alpha-quality=100`. Already installed (used for all placeholder generation). |

---

## Sources

- [Black Forest Labs API integration docs](https://docs.bfl.ml/api_integration/integration_guidelines) — EU endpoint (`api.eu.bfl.ai`), GDPR-compliant claim, polling pattern, model names — HIGH confidence
- [Black Forest Labs EU-Startups profile](https://www.eu-startups.com/2025/12/used-by-adobe-canva-and-meta-germanys-black-forest-labs-lands-e258-million-to-scale-its-visual-ai-platform/) — Freiburg HQ, German company, $300M round — HIGH confidence
- [fal.ai BiRefNet model page](https://fal.ai/models/fal-ai/birefnet) — endpoint `fal-ai/birefnet`, PNG+alpha output, v2 available — HIGH confidence
- [fal.ai FLUX model hub](https://fal.ai/flux) — FLUX Dev/Pro/Schnell/Kontext/1.1 availability, pricing context — MEDIUM confidence (verify endpoints via API docs before use)
- [fal.ai privacy policy](https://fal.ai/legal/privacy-policy) — US-domiciled data processing confirmed — HIGH confidence
- [web-sys Window struct docs](https://docs.rs/web-sys/latest/web_sys/struct.Window.html) — `Window` feature covers requestAnimationFrame, matchMedia, innerWidth/Height, addEventListener — HIGH confidence
- [ImageMagick WebP encoding options](https://imagemagick.org/script/webp.php) — lossless + alpha-quality support confirmed — HIGH confidence
- [WebP vs AVIF 2026 benchmark (pixotter)](https://pixotter.com/blog/webp-vs-avif/) — AVIF ~47x slower encode, 20-30% smaller files — MEDIUM confidence (third-party benchmark)
- [BiRefNet vs rembg production comparison (DEV Community)](https://dev.to/om_prakash_3311f8a4576605/birefnet-vs-rembg-vs-u2net-which-background-removal-model-actually-works-in-production-4830) — BiRefNet edge quality advantage in production — MEDIUM confidence

---

*Stack research for: v1.4 Immersive Companion — room rollout + AI art pipeline*
*Researched: 2026-06-03*
