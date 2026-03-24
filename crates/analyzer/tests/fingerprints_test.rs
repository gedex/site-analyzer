use site_analyzer::FingerprintDb;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::PathBuf;

#[test]
fn test_load_db() {
    let db = FingerprintDb::load();
    assert!(
        db.fingerprints.len() > 100,
        "Should have many fingerprints"
    );
    assert!(!db.categories.is_empty());
}

#[test]
fn test_wordpress_fingerprint() {
    let db = FingerprintDb::load();
    let wp = db
        .fingerprints
        .iter()
        .find(|f| f.name == "WordPress")
        .expect("WordPress fingerprint should exist");
    assert!(
        wp.cats.contains(&1),
        "WordPress should be in CMS category (1)"
    );
    assert!(
        !wp.html_patterns.is_empty(),
        "WordPress should have HTML patterns"
    );
    assert!(
        !wp.implies.is_empty(),
        "WordPress should imply PHP and MySQL"
    );
}

/// Test that all individual technology JSON files are valid
#[test]
fn test_all_technology_files_valid() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let tech_dir = PathBuf::from(manifest_dir)
        .join("src")
        .join("fingerprints")
        .join("technologies");

    assert!(
        tech_dir.exists(),
        "Technologies directory should exist at {:?}",
        tech_dir
    );

    let mapping_file = tech_dir.join("_mapping.json");
    let mapping_content =
        fs::read_to_string(&mapping_file).expect("Should be able to read _mapping.json");
    let mapping: HashMap<String, String> =
        serde_json::from_str(&mapping_content).expect("_mapping.json should be valid JSON");

    let mut tech_names = HashSet::new();
    let mut file_count = 0;
    let mut errors = Vec::new();

    for entry in fs::read_dir(&tech_dir).expect("Should be able to read technologies directory") {
        let entry = entry.expect("Should be able to read directory entry");
        let path = entry.path();

        // Skip non-JSON files and special files
        if !path.extension().map(|e| e == "json").unwrap_or(false) {
            continue;
        }
        let filename = path.file_stem().unwrap().to_str().unwrap();
        if filename == "_mapping" {
            continue;
        }

        file_count += 1;

        // Get tech name from mapping, or use filename as fallback (same as build.rs)
        let tech_name = mapping.get(filename)
            .cloned()
            .unwrap_or_else(|| filename.to_string());

        // Check for duplicate technology names
        if !tech_names.insert(tech_name.clone()) {
            errors.push(format!("Duplicate technology name: {}", tech_name));
        }

        // Try to parse the JSON - validate structure
        let content = match fs::read_to_string(&path) {
            Ok(c) => c,
            Err(e) => {
                errors.push(format!("Failed to read {}: {}", path.display(), e));
                continue;
            }
        };

        // Parse as generic JSON first
        let parsed: Result<serde_json::Value, _> = serde_json::from_str(&content);
        match parsed {
            Ok(value) => {
                // Check it's an object
                if !value.is_object() {
                    errors.push(format!("{}: Root must be a JSON object", tech_name));
                    continue;
                }

                let obj = value.as_object().unwrap();

                // Validate 'cats' field exists and is an array
                match obj.get("cats") {
                    Some(cats) => {
                        if let Some(arr) = cats.as_array() {
                            if arr.is_empty() {
                                errors.push(format!("{}: 'cats' array is empty", tech_name));
                            }
                            // Check all category IDs are numbers
                            for (idx, cat) in arr.iter().enumerate() {
                                if let Some(cat_num) = cat.as_u64() {
                                    if cat_num == 0 || cat_num > 100 {
                                        errors.push(format!(
                                            "{}: Invalid category ID {} at index {}",
                                            tech_name, cat_num, idx
                                        ));
                                    }
                                } else {
                                    errors.push(format!(
                                        "{}: Category at index {} is not a number",
                                        tech_name, idx
                                    ));
                                }
                            }
                        } else {
                            errors.push(format!("{}: 'cats' must be an array", tech_name));
                        }
                    }
                    None => {
                        errors.push(format!("{}: Missing required 'cats' field", tech_name));
                    }
                }

                // Validate pattern fields if present
                if let Some(html) = obj.get("html") {
                    validate_pattern_field(&mut errors, &tech_name, "html", html);
                }
                if let Some(scripts) = obj.get("scripts") {
                    validate_pattern_field(&mut errors, &tech_name, "scripts", scripts);
                }
                if let Some(css) = obj.get("css") {
                    validate_pattern_field(&mut errors, &tech_name, "css", css);
                }
                if let Some(url) = obj.get("url") {
                    validate_pattern_field(&mut errors, &tech_name, "url", url);
                }

                // Validate keyed pattern fields (objects)
                if let Some(headers) = obj.get("headers") {
                    validate_keyed_patterns(&mut errors, &tech_name, "headers", headers);
                }
                if let Some(meta) = obj.get("meta") {
                    validate_keyed_patterns(&mut errors, &tech_name, "meta", meta);
                }
                if let Some(cookies) = obj.get("cookies") {
                    validate_keyed_patterns(&mut errors, &tech_name, "cookies", cookies);
                }
                if let Some(js) = obj.get("js") {
                    validate_keyed_patterns(&mut errors, &tech_name, "js", js);
                }

                // Validate implies field
                if let Some(implies) = obj.get("implies") {
                    if !implies.is_string() && !implies.is_array() {
                        errors.push(format!(
                            "{}: 'implies' must be a string or array",
                            tech_name
                        ));
                    }
                }

                // Validate excludes field
                if let Some(excludes) = obj.get("excludes") {
                    if !excludes.is_string() && !excludes.is_array() {
                        errors.push(format!(
                            "{}: 'excludes' must be a string or array",
                            tech_name
                        ));
                    }
                }
            }
            Err(e) => {
                errors.push(format!("{}: Invalid JSON - {}", tech_name, e));
            }
        }
    }

    // Report results
    if !errors.is_empty() {
        eprintln!(
            "\n❌ Found {} errors in technology files:\n",
            errors.len()
        );
        for error in &errors {
            eprintln!("  - {}", error);
        }
        panic!("Technology validation failed with {} errors", errors.len());
    }

    assert!(
        file_count > 100,
        "Should have validated many technology files, found {}",
        file_count
    );
    println!("✅ Successfully validated {} technology files", file_count);
}

/// Validate a pattern field (can be string or array of strings)
fn validate_pattern_field(
    errors: &mut Vec<String>,
    tech_name: &str,
    field_name: &str,
    value: &serde_json::Value,
) {
    if value.is_string() {
        // Single pattern - OK
    } else if let Some(arr) = value.as_array() {
        // Array of patterns
        for (idx, item) in arr.iter().enumerate() {
            if !item.is_string() {
                errors.push(format!(
                    "{}: '{}' array element at index {} must be a string",
                    tech_name, field_name, idx
                ));
            }
        }
    } else {
        errors.push(format!(
            "{}: '{}' must be a string or array of strings",
            tech_name, field_name
        ));
    }
}

/// Validate keyed pattern fields (must be objects with string values or arrays)
fn validate_keyed_patterns(
    errors: &mut Vec<String>,
    tech_name: &str,
    field_name: &str,
    value: &serde_json::Value,
) {
    if let Some(obj) = value.as_object() {
        for (key, val) in obj.iter() {
            if !val.is_string() && !val.is_array() {
                errors.push(format!(
                    "{}: '{}[\"{}\"]' must be a string or array",
                    tech_name, field_name, key
                ));
            }
            if let Some(arr) = val.as_array() {
                for (idx, item) in arr.iter().enumerate() {
                    if !item.is_string() {
                        errors.push(format!(
                            "{}: '{}[\"{}\"][{}]' must be a string",
                            tech_name, field_name, key, idx
                        ));
                    }
                }
            }
        }
    } else {
        errors.push(format!(
            "{}: '{}' must be an object",
            tech_name, field_name
        ));
    }
}

/// Test that all technology names in mapping are unique
#[test]
fn test_mapping_has_no_duplicates() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let mapping_file = PathBuf::from(manifest_dir)
        .join("src")
        .join("fingerprints")
        .join("technologies")
        .join("_mapping.json");

    let mapping_content =
        fs::read_to_string(&mapping_file).expect("Should be able to read _mapping.json");
    let mapping: HashMap<String, String> =
        serde_json::from_str(&mapping_content).expect("_mapping.json should be valid JSON");

    let mut tech_names = HashSet::new();
    let mut duplicates = Vec::new();

    for (filename, tech_name) in mapping.iter() {
        if !tech_names.insert(tech_name.clone()) {
            duplicates.push(format!("{} (from {})", tech_name, filename));
        }
    }

    if !duplicates.is_empty() {
        panic!(
            "Found duplicate technology names in mapping: {:?}",
            duplicates
        );
    }
}

/// Test that the generated merged file matches individual files
#[test]
fn test_generated_matches_individual_files() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let generated_file = PathBuf::from(manifest_dir)
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("generated")
        .join("technologies.json");

    // Load from generated file
    let generated_content = fs::read_to_string(&generated_file)
        .expect("Generated file should exist (run 'cargo build' first)");
    let db_from_generated = FingerprintDb::from_json(&generated_content);

    // Count should match
    let tech_dir = PathBuf::from(manifest_dir)
        .join("src")
        .join("fingerprints")
        .join("technologies");

    let file_count = fs::read_dir(&tech_dir)
        .expect("Should be able to read technologies directory")
        .filter(|e| {
            if let Ok(entry) = e {
                let path = entry.path();
                if let Some(name) = path.file_stem() {
                    return path.extension().map(|e| e == "json").unwrap_or(false)
                        && name != "_mapping";
                }
            }
            false
        })
        .count();

    assert_eq!(
        db_from_generated.fingerprints.len(),
        file_count,
        "Generated file should contain all technology files"
    );
}
