use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let src_dir = PathBuf::from(&manifest_dir).join("src").join("fingerprints");

    let technologies_dir = src_dir.join("technologies");
    let categories_file = src_dir.join("categories.json");
    let mapping_file = technologies_dir.join("_mapping.json");

    // Output to generated/ directory (at the workspace root level for easier access)
    let out_dir = PathBuf::from(&manifest_dir)
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("generated");

    fs::create_dir_all(&out_dir).expect("Failed to create generated directory");
    let output_file = out_dir.join("technologies.json");

    println!("cargo:rerun-if-changed={}", technologies_dir.display());
    println!("cargo:rerun-if-changed={}", categories_file.display());

    // Load filename -> tech name mapping (optional)
    // Only needed when filename differs from tech name (e.g., "Next.js.json" -> "Next.js")
    let mapping: HashMap<String, String> = if mapping_file.exists() {
        let mapping_content = fs::read_to_string(&mapping_file)
            .expect("Failed to read _mapping.json");
        serde_json::from_str(&mapping_content)
            .expect("Failed to parse _mapping.json")
    } else {
        HashMap::new()
    };

    // Load categories
    let categories_content = fs::read_to_string(&categories_file)
        .expect("Failed to read categories.json");
    let categories: serde_json::Value = serde_json::from_str(&categories_content)
        .expect("Failed to parse categories.json");

    // Scan technologies directory and build merged structure
    let mut technologies = serde_json::Map::new();
    let entries = fs::read_dir(&technologies_dir)
        .expect("Failed to read technologies directory");

    for entry in entries {
        let entry = entry.expect("Failed to read directory entry");
        let path = entry.path();

        // Skip non-JSON files and the mapping file
        if !path.extension().map(|e| e == "json").unwrap_or(false) {
            continue;
        }
        let filename = path.file_stem().unwrap().to_str().unwrap();
        if filename == "_mapping" || filename == "README" {
            continue;
        }

        // Get tech name from mapping, or use filename as fallback
        let tech_name = mapping.get(filename)
            .cloned()
            .unwrap_or_else(|| filename.to_string());

        // Read the technology JSON
        let content = fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("Failed to read {}: {}", path.display(), e));

        let tech_data: serde_json::Value = serde_json::from_str(&content)
            .unwrap_or_else(|e| panic!("Failed to parse {}: {}", path.display(), e));

        technologies.insert(tech_name.clone(), tech_data);
    }

    // Build final merged structure
    let tech_count = technologies.len();
    let mut merged = serde_json::Map::new();
    merged.insert("technologies".to_string(), serde_json::Value::Object(technologies));
    merged.insert("categories".to_string(), categories);

    let merged_json = serde_json::Value::Object(merged);

    // Write the merged file
    let output = serde_json::to_string_pretty(&merged_json)
        .expect("Failed to serialize merged JSON");

    fs::write(&output_file, output)
        .expect("Failed to write generated technologies.json");

    println!("cargo:warning=Build script: Successfully merged {} technology fingerprints", tech_count);
}
