# Fix for Missing "Actualizaciones Recientes" in TUI Release Notes

## Problem
The TUI version was missing the "Actualizaciones Recientes" (Recent Updates) section that exists in the Node.js version of release notes generation.

## Analysis
- **Node.js version** (`scripts/prepare-release-notes.js`): Lines 669-681 include recent updates from Monday.com tasks
- **TUI version** (`src/app.rs`): The `generate_raw_release_notes` function was missing this section
- The `MondayTask` struct in `src/types.rs` already includes an `updates` field with the necessary data

## Solution
Added the missing "Actualizaciones Recientes" section to the TUI release notes generation in `src/app.rs`:

### Changes Made

1. **Location**: `src/app.rs` around line 1760 (in the `generate_raw_release_notes` function)

2. **Added Code**:
```rust
// Recent updates (Actualizaciones Recientes)
if !task.updates.is_empty() {
    document.push_str("- **Actualizaciones Recientes**:\n");
    
    // Show the 3 most recent updates
    for update in task.updates.iter().take(3) {
        // Format the date from ISO string to DD/MM/YYYY format
        let date = if let Ok(parsed_date) = update.created_at.parse::<chrono::DateTime<chrono::Utc>>() {
            parsed_date.format("%d/%m/%Y").to_string()
        } else if let Some(date_part) = update.created_at.split('T').next() {
            // If we can't parse the full datetime, try to extract just the date part
            if let Ok(parsed_date) = chrono::NaiveDate::parse_from_str(date_part, "%Y-%m-%d") {
                parsed_date.format("%d/%m/%Y").to_string()
            } else {
                update.created_at.clone()
            }
        } else {
            update.created_at.clone()
        };
        
        let creator_name = update.creator.as_ref()
            .map(|c| c.name.as_str())
            .unwrap_or("Usuario");
        
        // Truncate the body to 100 characters max
        let body_preview = if update.body.len() > 100 {
            format!("{}...", &update.body[..100])
        } else {
            update.body.clone()
        };
        
        document.push_str(&format!("  - {} por {}: {}\n", date, creator_name, body_preview));
    }
}
```

3. **Insertion Point**: Added after the SupportBee links section and before the Related commits section

## Expected Result
Now the TUI-generated release notes will include the "Actualizaciones Recientes" section for each Monday.com task, matching the format of the Node.js version:

```markdown
### BUG. Correción de informe "Resumen anual de entradas por centro" (ID: 8817155664)

- **Estado**: active
- **Tablero**: TEC-DEV-Teixo (ID: 1013914950)
- **Grupo**: 1.112.00 - 20250416
- **Detalles**:
  - [column details]
- **Enlaces SupportBee**:
  - [supportbee links]
- **Actualizaciones Recientes**:
  - 09/04/2025 por SG: Detectado por parte de SC que hai outro informe que ten o mesmo problema ca este, polo que despo...
  - 09/04/2025 por SG: Gerrit: https://gerrit.teimas.com/c/teixo/+/27211...
- **Commits Relacionados**:
  - [related commits]
```

## Verification
- Compilation succeeds with `cargo check`
- The TUI now includes the same "Actualizaciones Recientes" information as the Node.js version
- Date formatting matches the expected DD/MM/YYYY format used in the Node.js version
- Content truncation (100 characters) matches the Node.js implementation 