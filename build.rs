use std::{fs, path::Path};

fn main() {
    let specs_dir = Path::new("specs");
    let out_dir = std::env::var("OUT_DIR").unwrap();
    let features_dir = Path::new(&out_dir).join("features");
    fs::create_dir_all(&features_dir).unwrap();

    println!("cargo:rerun-if-changed=specs/");

    let mut spec_files: Vec<_> = fs::read_dir(specs_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|x| x.to_str()) == Some("md"))
        .collect();
    spec_files.sort_by_key(|e| e.path());

    for entry in &spec_files {
        let path = entry.path();
        println!("cargo:rerun-if-changed={}", path.display());
        let content = fs::read_to_string(&path).unwrap();
        let stem = path.file_stem().unwrap().to_str().unwrap();
        let blocks = extract_gherkin_blocks(&content);

        for (i, block) in blocks.iter().enumerate() {
            let name = if blocks.len() == 1 {
                stem.to_string()
            } else {
                format!("{stem}-{}", i + 1)
            };
            let feature_content =
                format!("# Generated from specs/{stem}.md\n\n{}\n", block.trim_end());
            fs::write(
                features_dir.join(format!("{name}.feature")),
                feature_content,
            )
            .unwrap();
        }
    }
}

fn extract_gherkin_blocks(content: &str) -> Vec<String> {
    let mut blocks = Vec::new();
    let mut in_block = false;
    let mut current = String::new();

    for line in content.lines() {
        let trimmed = line.trim();
        if !in_block {
            if trimmed == "```gherkin" || trimmed == "```feature" {
                in_block = true;
                current.clear();
            }
        } else if trimmed == "```" {
            blocks.push(current.clone());
            in_block = false;
        } else {
            current.push_str(line);
            current.push('\n');
        }
    }
    blocks
}
