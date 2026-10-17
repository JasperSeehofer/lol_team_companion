# Assets from outside the project

What the app ships that it did not draw itself, where each file came from,
and on what footing. Champion portraits, champion icons and item icons are
not listed: they come from Riot's Data Dragon at run time through
`/img/champion/...` and `/img/item/...` (`src/server/images.rs`) and are
never committed.

## Interface icons (M152)

Lane position, objective and structure icons, served by the app at
`GET /img/ui/{kind}/{name}` (`src/server/ui_icons.rs`). The browser never
requests a third party: client code only names the app's own route.

- **Source:** CommunityDragon (`raw.communitydragon.org`), a community
  mirror of the game client's files.
- **Pinned patch directory:** `16.20` (its `content-metadata.json` read
  `16.20.8248524+branch.releases-16-20.content.release`). Not `latest`: on the
  fetch date `latest` already served different bytes for five of the PNGs
  (`atakhan_r`, `dragon_elder` and the three structure icons), so a pin is the
  only way the proxied file and the committed copy stay the same file.
- **Fetched:** 2026-10-17, with `curl`, every file HTTP 200.
- **Allow-list:** exactly the 21 files below (`ui_icons::ICONS`). Any other
  kind or name is 404; the upstream URL is assembled only from the list's
  constants and never from request text, and no request text reaches the
  filesystem (the committed copies are compiled in with `include_bytes!`).
- **How they are served:**
  - Position SVGs are always the committed, sanitised copy; they are never
    proxied, so no upstream markup reaches a browser. Sent with
    `X-Content-Type-Options: nosniff` and
    `Content-Security-Policy: default-src 'none'; style-src 'unsafe-inline'; sandbox`.
  - PNGs are fetched once per process from the pinned directory (5 s timeout),
    kept in memory and served with `nosniff`. On a failure, a timeout, or a
    body that is not a PNG (or over 256 KB), the committed copy is served and
    upstream is asked again after 15 minutes.
  - Both: an `ETag` from the bytes, `If-None-Match` answers 304,
    `Cache-Control: public, max-age=604800`.
  - The same files are also present as static files under `/ui/...` because
    `public/` is the site's asset directory; nothing in the app links there.
- **Sanitiser:** `scripts/ui-icons/sanitise.py` (Python standard library):
  `python3 -I scripts/ui-icons/sanitise.py IN.svg OUT.svg`. It keeps only the
  elements `svg`, `g`, `path`, `circle` and `rect` and only the attributes
  `fill`, `fill-rule`, `opacity`, `stroke`, `d`, `cx`, `cy`, `r`, `x`, `y`,
  `width`, `height`, `viewBox` and `transform`; it drops processing
  instructions (the upstream `glow.css` link), scripts, styles, ids, classes,
  `foreignObject`, `xlink` and every `href`, event handlers and `url(...)`
  values; it rewrites each `<polygon>` as an equivalent `<path>`; and every
  fill and stroke colour becomes `currentColor`, so the app colours the icons
  with its tokens (a CSS mask in `RoleLabel`) and never shows the client's
  gold. A unit test (`committed_svgs_are_sanitised`) re-checks the committed
  copies. The `-light` and `-red` upstream variants are not used.
- **Not used:** the match-history team-coloured set (no surface needs it
  yet), and `map11.png`. The map texture is a private measuring reference for
  the Rift schematic only: **it is never committed, shipped, traced or
  vectorised**, and nothing derived from its pixels is in the repository.

| Committed file | Upstream (pinned) | sha256 upstream | sha256 committed |
|---|---|---|---|
| `public/ui/position/top.svg` | `https://raw.communitydragon.org/16.20/plugins/rcp-fe-lol-static-assets/global/default/svg/position-top.svg` | `9cdaec5ffc26e530bb4a7a1cc9192a7610b25be0f8fd366c3605c1afd360f916` | `de07afbad109e7cc4bd6f14402a049f94fa2f53bedfcfe706edb5dcd155d032b` |
| `public/ui/position/jungle.svg` | `https://raw.communitydragon.org/16.20/plugins/rcp-fe-lol-static-assets/global/default/svg/position-jungle.svg` | `b5fc0cc5ce9225a85a621216df3e284942bfc49380da4fdf4bf1f4bff428ddda` | `a96794d395cd52713241c54f54dc801863a603c630eae3d257e814d743b33b11` |
| `public/ui/position/middle.svg` | `https://raw.communitydragon.org/16.20/plugins/rcp-fe-lol-static-assets/global/default/svg/position-middle.svg` | `df95d266bc4cafd94e25ba7cb49cc3b85f7c31f8a95f9d5e496affffa783c953` | `46c86354a9af13dbe92fc5ec43deb6aba47fabc30ee2cb00d92a860a8cd0bdc4` |
| `public/ui/position/bottom.svg` | `https://raw.communitydragon.org/16.20/plugins/rcp-fe-lol-static-assets/global/default/svg/position-bottom.svg` | `2ee0de0500e2336ac44227b8040f47f7321f3e5f72cbad740c225633985855b2` | `30aa06746d9b6846da37959f93506116a949fd8d7bef207e4b919ff93cdc2123` |
| `public/ui/position/utility.svg` | `https://raw.communitydragon.org/16.20/plugins/rcp-fe-lol-static-assets/global/default/svg/position-utility.svg` | `fdbb740a56418806862ad70a49754141e06ab919ffc3a6ba97a0a3b987875e10` | `ff790dbbc56bdc50f1680e0049bd5d1f699114fa9615d526605330e110aa7521` |
| `public/ui/objective/baron.png` | `https://raw.communitydragon.org/16.20/game/assets/ux/minimap/icons/baron.png` | `e1d208f19982f9c1bd6ac380879e4438f55ada15326663e142c38c607d861564` | same |
| `public/ui/objective/riftherald.png` | `https://raw.communitydragon.org/16.20/game/assets/ux/minimap/icons/riftherald.png` | `12ffad9928a29f17ee45e085db039ba2e4e066d960c54d998f2683af598fd5f8` | same |
| `public/ui/objective/grub.png` | `https://raw.communitydragon.org/16.20/game/assets/ux/minimap/icons/grub.png` | `fc227d339710e3f4cc7b829f76b373de3248d8fd6568e2d6b06bc2789b4dadbf` | same |
| `public/ui/objective/dragon.png` | `https://raw.communitydragon.org/16.20/game/assets/ux/minimap/icons/dragon.png` | `c6b67012bba9f5cb59eec4845e2672251acfa60114e7f6ca87e721850593db22` | same |
| `public/ui/objective/dragon_cloud.png` | `https://raw.communitydragon.org/16.20/game/assets/ux/minimap/icons/dragon_cloud.png` | `04d51eefcdec3906475e71eb7f7a6f69bbfde64fd6fe69a8f462837bd714b096` | same |
| `public/ui/objective/dragon_infernal.png` | `https://raw.communitydragon.org/16.20/game/assets/ux/minimap/icons/dragon_infernal.png` | `1cbdc179f139465b06480780bb739049f5e653a83f70655e718eb6a3fde0b0ad` | same |
| `public/ui/objective/dragon_mountain.png` | `https://raw.communitydragon.org/16.20/game/assets/ux/minimap/icons/dragon_mountain.png` | `0b154f13184920bb03a92f38e812b386962aa7deb46af2f82d143ad130431352` | same |
| `public/ui/objective/dragon_ocean.png` | `https://raw.communitydragon.org/16.20/game/assets/ux/minimap/icons/dragon_ocean.png` | `edeef83cc787d503d4039347e01728f9746de03b322c58d2be641568e004311e` | same |
| `public/ui/objective/dragon_hextech.png` | `https://raw.communitydragon.org/16.20/game/assets/ux/minimap/icons/dragon_hextech.png` | `bc555144cb6337254c229180aecde8a760f7a94114d4e23a1d0c14f4e2eee96e` | same |
| `public/ui/objective/dragon_chemtech.png` | `https://raw.communitydragon.org/16.20/game/assets/ux/minimap/icons/dragon_chemtech.png` | `c67ce233349c75714d4fee9cf6b108fee0756b14a9057e591739cf96a76e2127` | same |
| `public/ui/objective/dragon_elder.png` | `https://raw.communitydragon.org/16.20/game/assets/ux/minimap/icons/dragon_elder.png` | `1d33e64840ecef20947c513059498a963fdce910e30c8a3ba1875b28428ef5f8` | same |
| `public/ui/objective/atakhan_r.png` | `https://raw.communitydragon.org/16.20/game/assets/ux/minimap/icons/atakhan_r.png` | `16e3712addde86e9a1c490e9e8912558b2df29272dc452056ea7603b919268f8` | same |
| `public/ui/objective/atakhan_v.png` | `https://raw.communitydragon.org/16.20/game/assets/ux/minimap/icons/atakhan_v.png` | `14e1f83836afd390490418bc894df267d7aba06a0584ec600de01bd1fc9e97da` | same |
| `public/ui/structure/tower.png` | `https://raw.communitydragon.org/16.20/game/assets/ux/minimap/icons/icon_ui_tower_minimap.png` | `ea2bfd61e2db0f30ed17bf595f4cdebff6980a12a284e9c770d0c436a7e16aa2` | same |
| `public/ui/structure/inhibitor.png` | `https://raw.communitydragon.org/16.20/game/assets/ux/minimap/icons/icon_ui_inhibitor_minimap_v2.png` | `d03a5155c27137f5812f0b927ab3c413dccffd17dad1fbc81d5d524dff708db6` | same |
| `public/ui/structure/nexus.png` | `https://raw.communitydragon.org/16.20/game/assets/ux/minimap/icons/icon_ui_nexus_minimap_v2.png` | `07f20edbf6a5a477f2fc2edcb49fb4d8dd40052e13d1c180a2ba8d241cccad58` | same |

## Footing

- Team Companion is a registered Riot developer product, **app 834192**
  (`public/riot.txt`). The use of game assets rests on **Riot's "Legal Jibber
  Jabber" policy, section 2, Exception 3** (products with a valid Riot API key
  that follow the API terms), not on the free fan-project grant.
- The notice that policy asks for (section 6) is shown beside Riot's
  disclaimer in the footer of every page, on the privacy page and in the
  terms (`components::footer::RIOT_LJJ_NOTICE`).
- **CommunityDragon is not on the Developer Portal's list of approved
  assets** (Data Dragon, the Press Kit, TFT and LoR), and its own reuse terms
  are unconfirmed. Data Dragon has no position or objective icons.
- **Open:** the owner's question to Riot through the Developer Portal about
  using these CommunityDragon icons for product 834192 is pending. If Riot
  says no, delete `public/ui/`, the `/img/ui/` route and the icon in
  `RoleLabel`; the role letters stand on their own.
- No Riot logo is used anywhere.
