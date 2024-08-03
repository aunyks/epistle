use clap::{Arg, Command};
use markdown::{mdast::Node, to_mdast, Constructs, ParseOptions};
use regex::Regex;
use std::collections::HashMap;
use std::io::Write;
use std::path::Path;
use std::{fs, fs::File};
fn extract_file_path(input: &str) -> Option<String> {
    let re = Regex::new(r#"file:(?:"(.*?)"|(\S+))"#).expect("Couldn't create file path regex");
    re.captures(input).and_then(|caps| {
        caps.get(1)
            .or_else(|| caps.get(2))
            .map(|m| m.as_str().to_string())
    })
}

fn extract_output_files(node: &Node, extracted_files: &mut HashMap<String, String>) {
    match node {
        Node::Code(code_block) => {
            if let Some(meta) = &code_block.meta {
                if let Some(file_path) = extract_file_path(meta) {
                    if let Some(file_contents) = extracted_files.get_mut(&file_path) {
                        let additional_file_contents = ["\n", &code_block.value, "\n"].concat();
                        file_contents.push_str(&additional_file_contents);
                    } else {
                        extracted_files.insert(file_path, code_block.value.clone());
                    }
                }
            }
        }
        _ => {
            if let Some(children) = node.children() {
                for child in children {
                    extract_output_files(child, extracted_files);
                }
            }
        }
    }
}

fn merge_paths(dir: &String, file: &String) -> Result<String, String> {
    if Path::new(file).is_absolute() {
        return Err(String::from("file is absolute"));
    }
    Ok(Path::new(dir).join(file).to_string_lossy().into_owned())
}

fn main() {
	let matches = Command::new("epistle")
        .arg(
            Arg::new("input_file")
                .short('i')
                .long("input_file")
                .value_name("FILE")
                .required(true)
                .help("Input Markdown file path"),
        )
        .arg(
            Arg::new("output_dir")
                .short('o')
                .long("output_dir")
                .value_name("DIR")
                .required(true)
                .help("Output project directory path"),
        )
        .get_matches();

let input_file = matches
	.get_one::<String>("input_file")
	.expect("Couldn't find input file in CLI args");
let output_dir = matches
	.get_one::<String>("output_dir")
	.expect("Couldn't find output directory in CLI args");

let markdown_content = fs::read_to_string(input_file).expect("Failed to read input file");

let options = ParseOptions {
        constructs: Constructs {
            code_fenced: true,
            ..Constructs::default()
        },
        ..Default::default()
    };
let ast = to_mdast(&markdown_content, &options).expect("Failed to parse Markdown");

let mut extracted_files: HashMap<String, String> = HashMap::new();
extract_output_files(&ast, &mut extracted_files);

	for (file_path, file_contents) in extracted_files {
        if let Ok(ultimate_file_path) = merge_paths(output_dir, &file_path) {
            if let Some(parent_dir) = Path::new(&ultimate_file_path).parent() {
                fs::create_dir_all(parent_dir).expect("Couldn't create file parent directories");
            }

            let mut file = File::create(ultimate_file_path).expect("Couldn't create or open file");
            file.write_all(file_contents.as_bytes())
                .expect("Couldn't write to file");
        }
    }
}
