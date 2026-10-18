# M151 link check (read-only research pass, 2026-10-16)

Method: WebFetch, WebSearch and curl (raw HTML for meta tags, `file` on the og images for real pixel sizes). "Opened" = HTTP 200 and readable. Nothing here is Riot artwork; the in-game position icons stay a "do not copy" reference only.

## Corrections to identity-proposal.md section 1 (remembered details that were wrong or unverifiable)

1. **Lucide is 24x24 with a 2 px stroke, 1 px safe zone, round joins and caps. Not "16px grid, 1.5px stroke".** Phosphor's README says "Designed at 16 x 16px" with six weights. So the 16 px / 1.5 px figure is our own choice, not a Lucide rule. Aicher's grid is not documented on Wikipedia either.
2. **FiveThirtyEight no longer resolves**: fivethirtyeight.com redirects to abcnews.com/politics. Do not cite it as a live reference. Use Our World in Data or Stripe Press for lead-figure pages.
3. **Kowalski "Animations on the Web"**: the title was remembered. Live pages are animations.dev (course) and emilkowal.ski/ui/*. The curve `cubic-bezier(0.23,1,0.32,1)` was NOT found on a primary Kowalski page; it appears only in third-party "skill" summaries. Treat as UNVERIFIED as attributed to him.
4. **swisstopo "Zeichenschluessel"**: the real German name is "Zeichenerklaerung" (symbol key brochure). The URL guessed in my notes returned 404; the working PDF is below.
5. **The Pudding**: unreachable from this sandbox (connection failed). UNVERIFIED, do not cite.
6. **Lucide guide URL**: `lucide.dev/guide/design/icon-design-guide` (the URL given in the brief) returns 404, and so does `lucide.dev/docs/icon-design-guide` when fetched directly (search engines list it). The working page is `lucide.dev/contribute/icons/design-principles/` (and `.md`).

## 1. Field-sketch ink (one-stroke schematic)

| URL | opened? | verified details | caveats |
|---|---|---|---|
| https://map.geo.admin.ch | Yes (200) | SPA; the page text carries no styling detail. | Visual traits UNVERIFIED by this pass; needs a screenshot by the atelier (use the Bash browser tools, not me). |
| https://www.swisstopo.admin.ch/dam/en/sd-web/WxsMJ4yE7xeV/Zeichenerklaerung_2008_e.pdf | Yes (PDF, 1.7 MB) | Symbol key for national maps 1:25 000, 1:50 000, 1:100 000 (per search snippet). PDF metadata: made in Illustrator CS3 in 2008; swatches Weiss, Schwarz (100 K), Rot (100 M + 100 Y); Frutiger LT Pro Light and Bold; four embedded CMYK images. | Page content is compressed, so which feature uses which colour is UNVERIFIED. A 2024 edition ("Landeskarten 1:10 000 bis 1:1 Million") exists; German PDF: .../dam/de/sd-web/WxsMJ4yE7xeV/Zeichenerklaerung_2008_d.pdf. The saved PDF is at the tool-results path of this session if the atelier wants to render it. |
| https://www.usgs.gov/media/images/field-notes | Yes | "Field Notes", posted 9 Apr 2009, credit Clay Martin, USGS, Public Domain. Description says the USGS Field Records Collection holds field notebooks, sketches, annotated maps, geologic cross sections. | The page does not say whose notes or confirm what the photo shows. It is a pointer, not a sketch example. |
| https://www.usgs.gov/faqs/how-can-i-find-original-field-notes-made-usgs-geologists-and-geographers | Seen in search results only (not fetched) | Field Records Collection is in the USGS Library, Denver; topographer notes go through NGTOC (tnm_help@usgs.gov). | UNVERIFIED beyond the search snippet. |
| https://www.usgs.gov/media/images/old-time-topographer-work-alidade-and-plane-table | Seen in search results only | Title only: a topographer at an alidade and plane table. | UNVERIFIED content. Public-domain USGS imagery is a candidate for real examples of hand-drawn plane-table sketches. |
| Strava / FATMAP terrain styling | NOT fetched | UNVERIFIED | No URL checked; drop from references.md or fetch separately. |
| https://en.wikipedia.org/wiki/Otl_Aicher | Yes | Aicher made the 1972 pictograms "using a series of grid systems", drew on the 1964 Tokyo Games iconography, and used a bright palette (blue and white as Alpine tones, plus green, orange, silver). Later influenced the 1974 US DOT pictograms. | The page says nothing about 45/90 degree construction; that remembered claim is UNVERIFIED. |

## 2. Mark grid (9 marks, 16 px, 1.5 px stroke)

| URL | opened? | verified details | caveats |
|---|---|---|---|
| https://lucide.dev/contribute/icons/design-principles/ | Yes (the brief's URL 404s) | Rules: 24 x 24 canvas; at least 1 px safe zone; 2 px strokes, no mixed widths; round line joins; round line caps on open paths; "round joins do not replace corner rounding" (corners still need a radius). | Our 16 px / 1.5 px is a deliberate departure: 1.5/16 = 0.094 against Lucide 2/24 = 0.083, so ours is slightly heavier relative to size. Say so in the recipe card. Search snippets add: rectangles >= 8 px get 2 px radius, smaller 1 px; 2 px spacing between elements (from the spec, not re-fetched). |
| https://github.com/phosphor-icons/homepage (README) and https://phosphoricons.com | README yes; the site's page text was empty | README: "1,248 icons and counting", six weights (Thin, Light, Regular, Bold, Fill, Duotone), "Designed at 16 x 16px", MIT licence. | The actual SVGs use `viewBox="0 0 256 256"` (seen via unpkg), so the "16 px" claim is about design intent, not the file grid. Stroke width per weight UNVERIFIED. |
| https://en.wikipedia.org/wiki/Otl_Aicher | Yes | See section 1: grid systems, no numeric values. | Numeric grid UNVERIFIED. |
| swisstopo symbol key (above) | Yes | As above. | Whether symbols are paired with text labels UNVERIFIED (PDF unreadable here). |
| In-game position icons | Not fetched, by rule | Do-not-copy reference only. | No Riot artwork noted or downloaded. |

## 3. Lock-in stroke motion (220 ms ease-out)

| URL | opened? | verified details | caveats |
|---|---|---|---|
| https://emilkowal.ski/ui/you-dont-need-animations | Yes | UI animations "should generally stay under 300ms"; "a 180ms dropdown feels more responsive than a 400ms one"; frequent interactions (launcher opened hundreds of times a day) work best with no animation; keyboard-initiated actions that repeat often should never be animated. | Does not discuss easing. Direct support for 220 ms < 300 ms. Warning that applies to us: a pick lock-in repeated many times per draft may count as "frequent"; the atelier should justify it as the single signature moment. |
| https://animations.dev | Yes | Course page: theory module 8 lessons; "a set of custom easing curves that I use in my work" promised but not shown; covers CSS and Motion (Framer Motion). Page said enrolment reopens Tuesday 10 November. | No curve or ms value on the page. |
| https://sonner.emilkowal.ski | Yes (home page also sets og:image 1686 x 882) | Toast types: default, description, success, info, warning, error, action, promise, custom; props `visibleToasts`, `expand`; swipe direction follows position. | Stacking/hover-expand/duration behaviour not stated on this page; claims in the proposal about stacking are UNVERIFIED here. |
| Third-party summaries (skills.cat, skills.sh etc.) of the emilkowal-animations skill | Search results only | Ease-out as default; strong ease-out `cubic-bezier(0.23, 1, 0.32, 1)` for UI; 200 ms ease-out as standard transition; under 300 ms ceiling; drawers 500 ms. | Secondhand and all derived from one repo; the curve is NOT confirmed from Kowalski himself. Our token matches this curve, which is fine, but do not attribute to him. |
| https://developer.apple.com/design/human-interface-guidelines/motion | Opened but the body text was not returned | Via secondary sources only: "Make motion optional"; supplement visual feedback with haptics/audio; crossfade is the usual Reduce Motion substitute. | Direct quotes UNVERIFIED. The proposal's "cross-fade or instant, not removal of the state change" is plausible but not confirmed on Apple's page. |
| Linear selection behaviour | linear.app/docs/keyboard-shortcuts-and-navigation 404 | UNVERIFIED | Needs a live look or drop the reference. |

## 4. Lead-figure typography (one figure, one sentence, one action)

| URL | opened? | verified details | caveats |
|---|---|---|---|
| https://ourworldindata.org | Yes | Page leads with a tagline H1 ("Research and data to make progress against the world's largest problems."); key numbers sit inside headlines ("A 73% average decline in wildlife populations"); a small date label sits above each entry; counters shown as figure plus label ("14,087 charts", "508 articles"); each chart carries a source and licence line (CC BY). | Typefaces and colours not in the text, so UNVERIFIED. A screenshot is needed for the type scale. |
| https://press.stripe.com (stripe.press redirects here) | Yes | Header "Stripe Press", tagline "Ideas for progress."; catalogue is a vertical list of title then author with a rule under each row; book pages: large title, author, short synopsis, purchase buttons. | Colours and typefaces UNVERIFIED (text-only fetch). |
| https://fivethirtyeight.com | Redirects to abcnews.com/politics | Brand folded into ABC News; og image there is the ABC default 2000x2000 image. | Not usable as a live reference. Use archived article URLs only if the atelier finds them. |
| The Pudding | No (connection failed) | UNVERIFIED | Drop. |

## 5. Open Graph card (1200 x 630)

Pixel sizes below were measured on the actual image files fetched with curl.

| URL | opened? | verified details | caveats |
|---|---|---|---|
| https://linear.app | Yes | `og:image` = `https://linear.app/api/og/main?title=Linear&v=4` (generated per title); `twitter:card` = `summary_large_image`; `twitter:image` same URL. Image measured: PNG 1200 x 630. | No `og:image:width/height` meta tags present. Content of the picture not viewed by this pass. |
| https://stripe.com (redirected to /de) | Yes | `og:image` = `https://b.stripecdn.com/mkt-ssr-statics/assets/_next/static/media/stripe.37119086.jpg`; `twitter:card` = `summary_large_image`. Image measured: JPEG 2048 x 1024 (2:1, not 1.91:1). | Proposal's "Stripe card is 1200x630" would have been wrong. Picture content not viewed. |
| https://press.stripe.com | Yes | `og:image` = `https://images.stripeassets.com/fzn2n1nzq965/13zaWB3aiz08hqNr69CMW4/772586e910faef5d26d15e01edb9c010/social.png?q=80`; `summary_large_image`. Measured: PNG 1200 x 675 (16:9). | Picture content not viewed. |
| https://ourworldindata.org | Yes | `og:image` = `/default-thumbnail.png`, measured PNG 1200 x 630; `summary_large_image`. | Chart-specific cards not checked. |
| https://animations.dev | Yes | Declares `og:image:width` 1200, `og:image:height` 630, type image/png, plus twitter width/height. | A clean example of explicit meta declaration. |
| https://vercel.com/docs/og-image-generation | Yes | "Recommended OG image size: 1200x630 pixels"; `@vercel/og` uses Satori + Resvg to turn HTML/CSS into PNG; only ttf, otf, woff fonts (ttf/otf preferred); flexbox only, no `display: grid`; maximum bundle 500 KB including fonts and images; allow the OG route in robots.txt. | Applies to Vercel's stack, not ours; what matters is the 1200 x 630 and the "no grid, bundle small" lesson for any server-side renderer we write. |
| https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/customizing-your-repository/customizing-your-repositorys-social-media-preview | Yes | Image "should be a PNG, JPG, or GIF file under 1 MB"; at least 640 x 320, 1280 x 640 for best display (2:1). | GitHub uses 2:1, other sites 1.91:1; 1200 x 630 is safe for Discord/most. Discord's own rules UNVERIFIED. |

## 6. Engraved contour (treatment T2)

| URL | opened? | verified details | caveats |
|---|---|---|---|
| https://en.wikipedia.org/wiki/Eduard_Imhof | Yes | Fame rests on relief shading for school maps and atlases (1922 to 1973); first Swiss professor of cartography (1925); founded the Cartographic Institute, Zurich; directed the Schweizerischer Mittelschulatlas 1932 to 1976. | Says nothing about contour or engraving technique or colour. Visual traits UNVERIFIED; needs a map image. |
| swisstopo 1:25k sheet (search results: cordee.co.uk niesen-1227, mapscompany.com) | Search results only | A retailer states contour spacing 10 m, 20 m in mountains. | Retailer claim, not swisstopo; check against the official key. |
| https://api3.geo.admin.ch/static/images/legends/ch.swisstopo.landeskarte-grau-10_de_big.pdf | Search result only | Legend PDF for the grey national map. | Not opened. |

## 7. Paper share page (treatment C)

| URL | opened? | verified details | caveats |
|---|---|---|---|
| swisstopo printed sheets | See section 1 (symbol key PDF) | Key uses white, black and a single spot red in its colour swatches; Frutiger Light/Bold. | Paper colour and layout UNVERIFIED. |
| https://press.stripe.com | Yes | Rule-separated list rows; book pages have zine cover PDFs ("Zine Cover" links to a PDF) and a "Living cover". | The "print feel" is not confirmed from text alone; needs a screenshot. |

## Open items for the atelier

- Screenshots needed (this pass is text-only): map.geo.admin.ch, press.stripe.com, ourworldindata.org, the three og images (URLs above, curl them).
- Fetch or drop: Strava/FATMAP, Linear docs, Apple HIG body text, The Pudding.
- Verified numbers worth quoting: Lucide 24 px / 2 px / 1 px safe zone / round joins and caps; Kowalski under 300 ms and "180 ms beats 400 ms"; Vercel 1200 x 630; GitHub 1280 x 640 under 1 MB; Linear og 1200 x 630.
