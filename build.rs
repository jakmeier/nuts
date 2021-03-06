use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::Path;

const HEADER: &'static str = "<!-- Auto generated by build.rs + README_TEMPLATE.md -->";

fn main() -> std::io::Result<()> {
    if let Ok(_) = std::env::var("DOCS_RS") {
        return Ok(());
    }
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=README_TEMPLATE.md");
    if let Err(_) = std::env::var("REBUILD_README") {
        return Ok(());
    }
    let out = fs::File::create("README.md")?;
    let mut readme = BufWriter::new(out);

    writeln!(readme, "{}", HEADER)?;

    let template = fs::File::open("README_TEMPLATE.md")?;
    let template = BufReader::new(template);

    let cargo_version = env!("CARGO_PKG_VERSION");
    let docs_url = format!("https://docs.rs/nuts/{}/nuts/", cargo_version);
    let info = ReadmeInfo { docs_url };

    let mut dict = HashMap::new();
    find_snippets_in_directory("src", &mut dict)?;
    for line in template.lines() {
        let line = line?;
        if line.starts_with("@DOC ") {
            let (_, key) = line.split_at(5);
            if let Some(doc) = dict.get(key) {
                writeln!(readme, "{}", readme_transformation(&doc, &info))?;
            } else {
                writeln!(readme, "MISSING DOCS: {} not found", key)?;
            }
        } else {
            writeln!(readme, "{}", line)?;
        }
    }
    println!("dictioniary: {:?}", dict);
    Ok(())
}

#[derive(Debug)]
struct Snippet {
    raw: String,
    file_path: Vec<String>,
}

fn find_snippets_in_directory(
    dir: impl AsRef<Path>,
    dict: &mut HashMap<String, Snippet>,
) -> std::io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();

        let metadata = fs::metadata(&path)?;
        if metadata.is_file() && path.extension().map(|ext| ext == "rs").unwrap_or(false) {
            find_snippets_in_file(path, dict)?;
        } else if metadata.is_dir() {
            find_snippets_in_directory(path, dict)?;
        }
    }
    Ok(())
}

#[derive(PartialEq, Debug)]
enum SearchState {
    OutsideSnippet,
    InsideSnippet,
}
fn find_snippets_in_file(
    path: impl AsRef<Path>,
    dict: &mut HashMap<String, Snippet>,
) -> std::io::Result<()> {
    let file = fs::File::open(&path)?;
    let mut state = SearchState::OutsideSnippet;
    let mut key = None;
    let mut snippet = String::new();
    const START_MARKER: &'static str = "// @ START-DOC ";
    const END_MARKER: &'static str = "// @ END-DOC";
    for (line_number, line) in BufReader::new(file).lines().enumerate() {
        let line = line?;
        let line = line.trim();
        match state {
            SearchState::OutsideSnippet => {
                if line.starts_with(START_MARKER) {
                    state = SearchState::InsideSnippet;
                    let (_start, end) = line.split_at(START_MARKER.len());
                    key = Some(end.to_owned());
                }
            }
            SearchState::InsideSnippet => {
                if line.starts_with(END_MARKER) {
                    state = SearchState::OutsideSnippet;
                    dict.insert(
                        key.take().expect("No key?"),
                        Snippet {
                            raw: std::mem::take(&mut snippet),
                            file_path: path
                                .as_ref()
                                .iter()
                                .map(|s| s.to_str().unwrap().to_owned())
                                .collect(),
                        },
                    );
                    println!("cargo:rerun-if-changed={}", path.as_ref().to_str().unwrap());
                } else {
                    snippet += "\n";
                    let uncommented = if let Some(i) = line.find("/// ") {
                        line.split_at(i + 4).1
                    } else if let Some(i) = line.find("///") {
                        line.split_at(i + 3).1
                    } else if let Some(i) = line.find("//! ") {
                        line.split_at(i + 4).1
                    } else if let Some(i) = line.find("//!") {
                        line.split_at(i + 3).1
                    } else {
                        panic!(
                            "Doc comment for README cannot be parsed: file: {}:{}, doc comment: {:?}",
                            path.as_ref().to_str().unwrap(),
                            line_number+1,
                            line
                        )
                    };
                    snippet += uncommented;
                }
            }
        }
    }
    assert_eq!(state, SearchState::OutsideSnippet);
    Ok(())
}

struct ReadmeInfo {
    docs_url: String,
}

#[derive(Eq, PartialEq)]
enum DecodeState {
    Init,
    ClosedBracket,
}

fn readme_transformation(input: &Snippet, info: &ReadmeInfo) -> String {
    // Find all doc links, e.g.
    // [`set_status`](struct.ActivityId.html#method.set_status)
    // and then transform to absolute address
    let mut out = String::new();
    let mut state = DecodeState::Init;
    for c in input.raw.chars() {
        out.push(c);
        match c {
            ']' => state = DecodeState::ClosedBracket,
            '(' => {
                if state == DecodeState::ClosedBracket {
                    out.push_str(&info.docs_url);
                }
                state = DecodeState::Init;
            }
            _ => state = DecodeState::Init,
        }
    }

    out
}
