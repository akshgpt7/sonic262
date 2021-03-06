use std::io::prelude::*;
use std::path::PathBuf;

use clap::Clap;
use yaml_rust::Yaml;

fn extract_strings(yaml: Option<&Yaml>) -> Option<Vec<String>> {
    match yaml {
        Some(Yaml::Array(array)) => Some(
            array
                .iter()
                .map(|v| match v {
                    Yaml::String(s) => s.clone(),
                    _ => String::new(),
                })
                .collect(),
        ),
        _ => None,
    }
}

fn extract_frontmatter(contents: &str) -> Yaml {
    let yaml_start = contents.find("/*---").unwrap();
    let yaml_end = contents.find("---*/").unwrap();
    let text = contents
        .get(yaml_start + 5..yaml_end)
        .unwrap()
        .replace("\r\n", "\n")
        .replace("\r", "\n");
    let text = text.trim_matches('\n');
    let frontmatter = yaml_rust::YamlLoader::load_from_str(&text).unwrap();
    frontmatter.first().cloned().unwrap()
}

fn generate_includes(includes: Vec<String>, include_path: &PathBuf) -> String {
    let mut contents = String::new();
    for include in includes {
        let mut file = std::fs::File::open(include_path.join(include)).unwrap();
        let mut file_contents = String::new();
        file.read_to_string(&mut file_contents).unwrap();
        contents.push_str(&file_contents);
        contents.push('\n');
    }
    contents
}

fn process_file(test_path: &PathBuf, include_path: &PathBuf) {
    let mut test_file = std::fs::File::open(test_path).unwrap();
    let mut contents = String::new();
    test_file.read_to_string(&mut contents).unwrap();
    let frontmatter = extract_frontmatter(&contents);
    if let Yaml::Hash(h) = frontmatter {
        // let flags = extract_strings(h.get(&Yaml::String(String::from("flags"))));
        // let features = extract_strings(h.get(&Yaml::String(String::from("features"))));
        let mut includes =
            extract_strings(h.get(&Yaml::String(String::from("includes")))).unwrap_or_default();
        includes.push(String::from("assert.js"));
        includes.push(String::from("sta.js"));
        let mut include_contents = generate_includes(includes, include_path);
        include_contents.push_str(&contents);
        let mut final_file = tempfile::NamedTempFile::new().unwrap();
        final_file.write_all(include_contents.as_bytes()).unwrap();
        let node_process = std::process::Command::new("node")
            .arg(final_file.path())
            .output()
            .unwrap();
    }
}

#[derive(Clap)]
#[clap(version = "0.1.0", author = "Ujjwal Sharma <ryzokuken@disroot.org>")]
struct Opts {
    #[clap(long)]
    root_path: Option<PathBuf>,
    #[clap(long)]
    test_path: Option<PathBuf>,
    #[clap(long)]
    include_path: Option<PathBuf>,
}

fn main() {
    let args = Opts::parse();
    let root_path = args.root_path;
    let test_path = args
        .test_path
        .unwrap_or_else(|| root_path.clone().unwrap().join("test"));
    let include_path = args
        .include_path
        .unwrap_or_else(|| root_path.unwrap().join("harness"));
    if test_path.is_file() {
        process_file(&test_path, &include_path);
    } else {
        for entry in walkdir::WalkDir::new(test_path) {
            let ent = entry.unwrap();
            if ent.metadata().unwrap().is_file() {
                process_file(&ent.into_path(), &include_path);
            }
        }
    }
}
