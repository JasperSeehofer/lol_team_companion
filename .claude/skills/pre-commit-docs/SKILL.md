---
name: pre-commit-docs
description: Check whether CHANGELOG.md, TODO.md, CLAUDE.md, and INSTRUCTIONS.md need updating before a commit. MUST be used before every commit in this project. Trigger when the user asks to commit, says "check docs", "pre-commit", or when you are about to create a git commit. Also trigger if user says "/pre-commit-docs" explicitly.
---

# Pre-Commit Documentation Check

Review the current changes and determine whether the project's 4 key markdown files need updating before the commit is created.

## Files to Check

| File | What to look for |
|------|-----------------|
| `CHANGELOG.md` | Does the `[Unreleased]` section have entries for every user-visible change in the diff? New features → `### Added`, bug fixes → `### Fixed`, modifications → `### Changed`. |
| `TODO.md` | Should any open `- [ ]` items be marked `- [x]`? Should new items be added for bugs discovered, features deferred, or work explicitly punted? |
| `CLAUDE.md` | Were new gotchas or patterns discovered (new numbered rules)? New component patterns, DB query patterns, or server function conventions? New sections needed (routes table, directory layout)? |
| `INSTRUCTIONS.md` | Does the status line at the top still reflect reality? This file is mostly static — only flag if the project scope or section status changed significantly. |

## Process

### Step 1: Gather the diff

Read the staged changes. If nothing is staged, read unstaged changes against HEAD.

```bash
# Prefer staged changes (what will actually be committed)
git diff --cached --stat
git diff --cached
# Fallback: unstaged changes
git diff --stat
git diff
```

Also check recent commits if doing a multi-commit review:
```bash
git log --oneline -5
```

### Step 2: Read all 4 files

Read the current contents of:
- `CHANGELOG.md`
- `TODO.md`
- `CLAUDE.md`
- `INSTRUCTIONS.md`

### Step 3: Analyze and report

For each file, determine one of:
- **Up to date** — no changes needed
- **Needs update** — with specific suggestions of what to add/change/check off

Output a checklist like:

```
## Pre-Commit Docs Check

- [x] CHANGELOG.md — up to date
- [ ] TODO.md — mark "Interactive tree graph" as complete (tree_graph.rs changes)
- [ ] CLAUDE.md — add rule 56: new pattern for X discovered in this session
- [x] INSTRUCTIONS.md — no changes needed
```

### Step 4: Suggest edits

For each file that needs updating, provide the specific text to add or change. Use the exact format each file expects:

**CHANGELOG.md** — Keep a Changelog format:
```markdown
## [Unreleased]

### Added
- Description of new feature

### Fixed
- Description of bug fix
```

**TODO.md** — Checkbox format, placed in the correct priority section:
```markdown
- [x] **Item name**: Description of completion
- [ ] **New item**: Description of new work
```

**CLAUDE.md** — Numbered gotcha format (continue from last number):
```markdown
56. **Short title** — Explanation of the gotcha or pattern, with code example if helpful.
```

**INSTRUCTIONS.md** — Update status line only:
```
STATUS: All sections implemented as of YYYY-MM-DD (Sections 1-N). ...
```

### Step 5: Ask for confirmation

Present the suggested changes and ask the user whether to:
1. Apply all suggestions
2. Apply selectively
3. Skip (docs are fine as-is)

Then proceed with the commit.

## Important

- Do NOT skip this check. Every commit in this project should have its docs reviewed.
- Be conservative — only suggest changes that are clearly warranted by the diff.
- Don't add changelog entries for internal refactors, tooling changes, or non-user-visible work unless they affect the dev workflow documented in CLAUDE.md.
- When in doubt about whether something belongs in CLAUDE.md, ask — the user prefers a lean reference doc over an exhaustive one.
