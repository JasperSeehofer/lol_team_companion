# M156: Riot Developer Portal question (draft)

## Context for Jasper

- This is a draft. You send it yourself from the Developer Portal for product **834192**; nothing has been sent.
- It goes out with focus row 7's portal text (Team Companion launch owner steps), **due 2026-10-20**.
- Question 2 is the CommunityDragon icon ask from M152, in the words of its card: role and objective icons, and a map derived from the texture.
- Unsure: whether Riot wants Scout registered as its own product or under 834192. The message asks.
- Unsure: Riot's overlay rules are about live advantage. Offline review of your own replay is not covered either way, so we are asking rather than assuming.
- The 12-month figure is `MATCH_CACHE_RETENTION_DAYS=365`. Change both together if you change it.
- Edit freely. The body must stay at 250 words or fewer, and the product is only ever "Team Companion".

## Message

**Subject:** Product 834192 (Team Companion): offline review of a player's own replays, and CommunityDragon icons

Hello,

Team Companion (product 834192) is a private coaching workspace for one amateur team. We have two questions before we build anything.

1. Offline replay review. We are considering a separate tool, free and open source (AGPL-3.0), that a player runs on their own Windows PC on a replay they downloaded themselves. It uses only the official local Replay API (127.0.0.1:2999, which the player switches on with EnableReplayApi=1) and screen capture of the replay's minimap or zoomed-out view. From these it extracts per-second champion positions, ward placements and recalls for post-game review. It does no memory reading, no injection and no .rofl decoding, and it never runs during a live match. The player would then upload the derived tracks to their own team's private workspace. The tracks include the opponents' positions, because those are in the player's replay. They would be visible only to that team, deletable, and kept no longer than our match cache (12 months). Is this acceptable? Should the tool be registered under this product or separately?

2. Icons. May Team Companion reuse CommunityDragon's position (role) and objective icons, served from our own domain alongside Data Dragon? Would a map derived from the map texture be allowed? Today we only measure coordinates and draw our own schematic.

Thank you for your time.

Kind regards,
Jasper Seehofer, Team Companion
