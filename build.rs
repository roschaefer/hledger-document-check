use std::path::Path;

fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    markdown_to_cucumber::specs::generate_features(Path::new("specs"), Path::new(&out_dir));
}
