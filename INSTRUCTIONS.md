League of Legends Team Companion App - Complete Specification

STATUS: All sections implemented as of 2026-03-11 (Sections 1-10). See TODO.md for completed items and CHANGELOG.md for version history.

OVERVIEW
Your app has 6 main sections:

Teams - Create/manage teams, rosters, and join requests
Drafts - Advanced tree-based draft planning with live game navigation
Stats - Track team performance via League API integration
Game Plans - Strategy planning tied to specific matchups and drafts
Postgame - Feedback and analysis linked to actual matches
Profile - User auth, settings, and champion pools


DETAILED FEATURE BREAKDOWN
TEAMS SECTION
Core Features:

Team creation (owner sets name)
Join request system with notifications
Main roster (1 player per role)
Substitute roster (multiple players per role)
Owner-only editing
Overall team stats display

To-Dos: Team creation form → Team editing (owner-only) → Roster assignment UI → Substitute management → Join request notifications → Accept/decline functionality → Permission controls
DRAFTS SECTION
What You Have: Full draft creation, opponent assignment, rating system, visual client display, searchable champion grid, drag-and-drop, comments, save/load
What's New - Draft Trees:

Branching paths based on enemy actions
Full tree visualization in edit mode with zoom
Step-by-step navigator for live games
Improvisation mode with tagging
Minimal required fields + optional notes per node

To-Dos: Redesign data model for trees → Tree visualization → Zoom functionality → Branch editing → Live game navigator → Improvisation tagging → Migration of existing drafts
STATS SECTION
Core Features:

Auto-sync from League API (few times daily)
Manual refresh option
Filter by date range and opponent
Archive all match data in database
Only include games with all 5 roster players

To-Dos: API authentication → Match filtering logic → Database schema for stats → Auto-sync scheduling → Manual refresh button → Filter UI → Pagination
GAME PLAN SECTION
Core Features:

Create plans tied to specific matchups (your 5 champs vs their 5)
Macro strategy (team-wide, visible to all)
Role-specific strategy (tailored to each player)
Link to specific draft
Template-based auto-generation with customization

To-Dos: Game plan form → Matchup selection → Macro strategy section → Role-specific sections (×5) → Template system → Basic matchup analysis → Customization interface
POSTGAME SECTION
Core Features:

Linked to actual match from stats
Linked to original game plan and draft
Structured feedback tied to game plan points
Open-ended notes
Visible to entire team
Pattern analysis (what works/doesn't)

To-Dos: Postgame form → Match linking → Structured feedback fields → Open notes section → Display/review page → Pattern analysis → Comparison view (planned vs actual)
PROFILE SECTION
Core Features:

Sign up/login
Riot OAuth
Champion pools (per role, editable by player only)
Team memberships
Personal stats
Notification & display preferences

To-Dos: Auth system → OAuth integration → Profile page → Champion pool UI → Pools visibility → Settings panels → Data persistence

READY-TO-USE CLAUDE CODE PROMPTS
Here are 8 focused prompts you can give Claude Code directly:
PROMPT 1 - Authentication
Build authentication for the League of Legends Team Companion app:
- Sign up/login with username and password
- Riot OAuth integration
- User profile page with linked account
- Teams list, notification settings, display preferences
- Users database with proper password hashing
PROMPT 2 - Team Management
Build complete team management system:
- Team creation form and editing (owner-only)
- Join request system with notifications
- Main roster (5 roles, one player each)
- Substitute roster (multiple per role)
- Drag-and-drop role assignment
- Permission checks (owner-only editing)
PROMPT 3 - Draft Tree System
Build advanced draft planning with tree structure:
- Redesign drafts as decision trees (nodes = decisions, branches = paths)
- Full tree visualization with zoom in/out
- Edit mode: add branches, edit nodes, add notes
- Live game navigator: step-by-step with branch highlighting
- Improvisation mode: create new branches during game, tag as "improvised"
- Save and learn from improvised branches
PROMPT 4 - League API & Stats
Build statistics system with League API:
- Riot API authentication and error handling
- Pull match history, filter for all-5-player games
- Extract all available stats (KDA, gold, vision, objectives)
- Auto-sync few times daily, plus manual refresh
- Database schema to archive all match data
- Stats dashboard with date/opponent filters
- Pagination for large datasets
PROMPT 5 - Game Plan System
Build game planning system:
- Create game plans for specific matchups (your 5 champs vs enemy 5)
- Macro strategy section (team-wide, visible to all)
- 5 role-specific strategy sections (only show relevant to each player)
- Template-based auto-generation: suggest win conditions and power spikes
- Full customization after generation
- Link to specific draft
- Save/load/edit/version control
PROMPT 6 - Postgame Analysis
Build postgame feedback system:
- Link to actual match from stats (League API)
- Link to original game plan and draft used
- Structured feedback fields tied to game plan points
- Open-ended notes section
- Display page showing: planned strategy → actual stats → team feedback
- Pattern analysis: identify what works and what doesn't
- Comparison view: plan vs actual results
PROMPT 7 - Champion Pools & Profile
Build champion pool system and profile enhancements:
- Per-role champion pools for each player
- Only the player can edit their own pool
- Team can view all players' pools
- Use pools in draft planning UI
- Profile enhancements: show pools, personal stats, teams, settings
- Integrate with game plan and draft suggestions
PROMPT 8 - Full Integration & Polish
Build final integration and UI:
- Main navigation (Teams, Drafts, Stats, Game Plans, Postgame, Profile)
- Consistent header with user info and notifications
- Dashboard showing teams, recent drafts, alerts, stats summary
- Notification system with badge and dropdown
- Responsive design for desktop/mobile
- Dark mode toggle (optional)
- Error handling and graceful API failures
- Performance optimization and caching

RECOMMENDED BUILD ORDER

Auth & Profile (Prompt 1) - Foundation
Team Management (Prompt 2) - Core functionality
Champion Pools (Prompt 7) - Quick win, enables other features
Draft Trees (Prompt 3) - Your unique feature
Stats System (Prompt 4) - Data backbone
Game Plans (Prompt 5) - Strategy layer
Postgame (Prompt 6) - Learning loop
Integration & Polish (Prompt 8) - Finishing touches
