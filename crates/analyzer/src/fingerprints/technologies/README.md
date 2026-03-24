# Technology Fingerprints

This directory contains individual JSON files for each technology fingerprint.

## Structure

- Each technology has its own `.json` file (e.g., `WordPress.json`, `React.json`)
- `_mapping.json` (optional) overrides filename → technology name mapping
- At compile time, `build.rs` merges all files into a single `generated/technologies.json`

## Adding a New Technology

1. Create a new JSON file in this directory:
   ```bash
   cat > "MyTechnology.json" << 'EOF'
   {
     "cats": [59],
     "website": "https://example.com",
     "description": "Description of the technology",
     "headers": {
       "X-Custom-Header": "pattern"
     },
     "html": "regex-pattern",
     "scripts": ["pattern1", "pattern2"],
     "meta": {
       "generator": "^MyTech ([\\d.]+)\\;version:\\1"
     },
     "implies": ["Node.js"]
   }
   EOF
   ```

2. (Optional) If the filename differs from the technology name:
   ```bash
   # Edit _mapping.json to add:
   # "MyTechnology": "My Technology"
   ```

3. Rebuild the project:
   ```bash
   cargo build
   ```

   The technology will be automatically included!

## Editing an Existing Technology

Just edit the corresponding `.json` file directly! The changes will be picked up on the next build.

Example - add a new HTML pattern to WordPress:

```bash
# Edit the file
vim WordPress.json

# Rebuild
cargo build
```

## Removing a Technology

1. Delete the JSON file:
   ```bash
   rm OldTech.json
   ```

2. (Optional) Remove from `_mapping.json` if present

3. Rebuild: `cargo build`

## Pattern Format

Patterns use the Wappalyzer format:

```
regex_pattern\\;version:\\1\\;confidence:75
```

- Everything before the first `\\;` is the regex (case-insensitive by default)
- `version:\\1` — version capture (\\1, \\2 are regex capture groups)
- `confidence:N` — 0-100, default 100 when omitted
- Empty pattern (`""`) = presence check (always matches)

## Category IDs

Common categories:
- `1` = CMS
- `11` = Blogs
- `12` = JavaScript Frameworks
- `18` = Web Frameworks
- `22` = Web Servers
- `27` = Programming Languages
- `59` = JavaScript Libraries

See `../categories.json` for the full list.
