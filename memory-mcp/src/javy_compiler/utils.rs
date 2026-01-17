//! Utility functions for JavaScript validation and caching

use anyhow::{anyhow, Result};
use std::path::Path;

/// Validate JavaScript syntax (basic validation)
pub fn validate_js_syntax(js_source: &str) -> Result<()> {
    // Basic syntax validation - check for balanced braces, brackets, parentheses
    let mut brace_count = 0;
    let mut bracket_count = 0;
    let mut paren_count = 0;
    let mut in_string = false;
    let mut escape_next = false;

    for (i, ch) in js_source.char_indices() {
        if escape_next {
            escape_next = false;
            continue;
        }

        if ch == '\\' {
            escape_next = true;
            continue;
        }

        if ch == '"' || ch == '\'' {
            in_string = !in_string;
            continue;
        }

        if in_string {
            continue;
        }

        match ch {
            '{' => brace_count += 1,
            '}' => {
                if brace_count == 0 {
                    return Err(anyhow!("Unmatched closing brace at position {}", i));
                }
                brace_count -= 1;
            }
            '[' => bracket_count += 1,
            ']' => {
                if bracket_count == 0 {
                    return Err(anyhow!("Unmatched closing bracket at position {}", i));
                }
                bracket_count -= 1;
            }
            '(' => paren_count += 1,
            ')' => {
                if paren_count == 0 {
                    return Err(anyhow!("Unmatched closing parenthesis at position {}", i));
                }
                paren_count -= 1;
            }
            _ => {}
        }
    }

    if brace_count != 0 {
        return Err(anyhow!("Unmatched opening braces: {}", brace_count));
    }
    if bracket_count != 0 {
        return Err(anyhow!("Unmatched opening brackets: {}", bracket_count));
    }
    if paren_count != 0 {
        return Err(anyhow!("Unmatched opening parentheses: {}", paren_count));
    }

    Ok(())
}

/// Check if a WASM file is valid by checking magic bytes and minimum size
pub fn is_valid_wasm_file(path: &Path) -> bool {
    if let Ok(mut file) = std::fs::File::open(path) {
        let mut magic = [0u8; 4];
        if std::io::Read::read_exact(&mut file, &mut magic).is_ok() {
            return &magic == b"\0asm" && file.metadata().map(|m| m.len() > 100).unwrap_or(false);
        }
    }
    false
}

/// Generate a cache key for the JavaScript source
pub fn generate_cache_key(js_source: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    js_source.hash(&mut hasher);
    format!("js_{:x}", hasher.finish())
}
