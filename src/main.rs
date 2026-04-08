use std::cell::RefCell;
use std::cmp::Reverse;
use std::path::{Path, PathBuf};

use clap::{Parser as ClapParser, ValueEnum};
use rayon::prelude::*;
use walkdir::WalkDir;

#[derive(Clone, ValueEnum)]
enum ReportType {
    Method,
    Class,
    Both,
}

#[derive(ClapParser)]
#[command(name = "class-method")]
#[command(about = "PHP クラス/メソッド行数レポート")]
struct Cli {
    /// 表示件数
    #[arg(short = 'n', default_value = "10")]
    top_n: usize,

    /// method / class / both
    #[arg(short = 't', default_value = "both")]
    report_type: ReportType,

    /// 対象ディレクトリ
    #[arg(default_value = "src")]
    target: PathBuf,
}

struct Entry {
    file: String,
    name: String,
    lines: usize,
}

struct FileResult {
    methods: Vec<Entry>,
    classes: Vec<Entry>,
}

thread_local! {
    static PARSER: RefCell<tree_sitter::Parser> = RefCell::new({
        let mut p = tree_sitter::Parser::new();
        p.set_language(&tree_sitter_php::LANGUAGE_PHP.into()).unwrap();
        p
    });
}

fn analyze_file(path: &Path) -> Option<FileResult> {
    let source = std::fs::read_to_string(path)
        .map_err(|e| eprintln!("Warning: {}: {e}", path.display()))
        .ok()?;
    let tree = PARSER.with(|p| p.borrow_mut().parse(&source, None))?;

    let mut methods = Vec::new();
    let mut classes = Vec::new();
    let path_str = path.to_string_lossy().into_owned();

    collect_nodes(tree.root_node(), &source, &path_str, &mut methods, &mut classes);

    Some(FileResult { methods, classes })
}

fn collect_nodes(
    node: tree_sitter::Node,
    source: &str,
    file: &str,
    methods: &mut Vec<Entry>,
    classes: &mut Vec<Entry>,
) {
    match node.kind() {
        "class_declaration" | "trait_declaration" | "interface_declaration"
        | "enum_declaration" | "anonymous_class" => {
            let name = extract_name(node, source);
            let start_row = effective_start_row(node);
            let lines = node.end_position().row - start_row + 1;
            let start_line = start_row + 1;

            classes.push(Entry {
                file: format!("{file}:{start_line}"),
                name,
                lines,
            });

            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                collect_nodes(child, source, file, methods, classes);
            }
        }
        "method_declaration" => {
            let name = extract_name(node, source);
            let start_row = effective_start_row(node);
            let lines = node.end_position().row - start_row + 1;
            let start_line = start_row + 1;

            methods.push(Entry {
                file: format!("{file}:{start_line}"),
                name,
                lines,
            });
        }
        _ => {
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                collect_nodes(child, source, file, methods, classes);
            }
        }
    }
}

/// PHP 8 アトリビュート (#[...]) を除いた実効開始行を返す (0-indexed)
fn effective_start_row(node: tree_sitter::Node) -> usize {
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if child.kind() != "attribute_list" && child.kind() != "attribute_group" {
            return child.start_position().row;
        }
    }
    node.start_position().row
}

fn extract_name(node: tree_sitter::Node, source: &str) -> String {
    node.child_by_field_name("name")
        .and_then(|n| n.utf8_text(source.as_bytes()).ok())
        .unwrap_or("(anonymous)")
        .to_string()
}

fn print_table(label: &str, top_n: usize, entries: &[Entry]) {
    println!();
    println!("=== {label} Top {top_n} ===");
    println!();
    if entries.is_empty() {
        println!("(no results)");
        return;
    }
    let shown: Vec<&Entry> = entries.iter().take(top_n).collect();
    let file_width = shown.iter().map(|e| e.file.len()).max().unwrap_or(4).max(4);
    println!("{:<6}  {:<file_width$}  {}", "Lines", "File", "Name");
    println!("{:<6}  {:<file_width$}  {}", "-----", "----", "----");
    for e in &shown {
        println!("{:<6}  {:<file_width$}  {}", e.lines, e.file, e.name);
    }
}

fn main() {
    let cli = Cli::parse();

    if !cli.target.exists() {
        eprintln!("Error: {} が見つかりません", cli.target.display());
        std::process::exit(1);
    }

    eprintln!("Analyzing {} ...", cli.target.display());

    let php_files: Vec<PathBuf> = WalkDir::new(&cli.target)
        .into_iter()
        .filter_map(|e| e.map_err(|err| eprintln!("Warning: {err}")).ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "php"))
        .map(|e| e.into_path())
        .collect();

    let results: Vec<FileResult> = php_files
        .par_iter()
        .filter_map(|path| analyze_file(path))
        .collect();

    let mut all_methods: Vec<Entry> = Vec::new();
    let mut all_classes: Vec<Entry> = Vec::new();

    for r in results {
        all_methods.extend(r.methods);
        all_classes.extend(r.classes);
    }

    all_methods.sort_unstable_by_key(|e| Reverse(e.lines));
    all_classes.sort_unstable_by_key(|e| Reverse(e.lines));

    if matches!(cli.report_type, ReportType::Class | ReportType::Both) {
        print_table(
            "ExcessiveClassLength (クラス行数)",
            cli.top_n,
            &all_classes,
        );
    }

    if matches!(cli.report_type, ReportType::Method | ReportType::Both) {
        print_table(
            "ExcessiveMethodLength (メソッド行数)",
            cli.top_n,
            &all_methods,
        );
    }

    println!();
}
