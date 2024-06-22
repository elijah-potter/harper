use std::path::{Path, PathBuf};

use anyhow::format_err;
use ariadne::{Color, Label, Report, ReportKind, Source};
use clap::Parser;
use harper_core::{remove_overlaps, Document, FullDictionary, LintGroup, LintGroupConfig, Linter};
use harper_tree_sitter::TreeSitterParser;

#[derive(Debug, Parser)]
struct Args {
    /// The file you wish to grammar check.
    file: PathBuf,
    /// Whether to merely print out the number of errors encountered, without
    /// further details.
    #[arg(short, long)]
    count: bool
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let source = std::fs::read_to_string(&args.file)?;

    let parser = TreeSitterParser::new_from_language_id(
        filename_to_filetype(&args.file).ok_or(format_err!("Could not detect filetype."))?
    )
    .ok_or(format_err!("Could not detect language ID."))?;

    let doc = Document::new(&source, Box::new(parser));

    let mut linter = LintGroup::new(
        LintGroupConfig::default(),
        FullDictionary::create_from_curated()
    );
    let mut lints = linter.lint(&doc);

    if args.count {
        println!("{}", lints.len());
        return Ok(());
    }

    if lints.is_empty() {
        println!("No lints found");
        return Ok(());
    }

    remove_overlaps(&mut lints);

    let primary_color = Color::Magenta;

    let filename = args
        .file
        .file_name()
        .map(|s| s.to_string_lossy().into())
        .unwrap_or("<file>".to_string());

    let mut report_builder = Report::build(ReportKind::Advice, &filename, 0);

    for lint in lints {
        report_builder = report_builder.with_label(
            Label::new((&filename, lint.span.into()))
                .with_message(lint.message)
                .with_color(primary_color)
        );
    }

    let report = report_builder.finish();
    report.print((&filename, Source::from(source)))?;

    std::process::exit(1);
}

/// Convert a provided path to a corresponding Language Server Protocol file
/// type.
///
/// Note to contributors: try to keep this in sync with
/// [`TreeSitterParser::new_from_language_id`]
fn filename_to_filetype(path: &Path) -> Option<&'static str> {
    Some(match path.extension()?.to_str()? {
        "rs" => "rust",
        "ts" => "typescript",
        "tsx" => "typescriptreact",
        "js" => "javascript",
        "jsx" => "javascriptreact",
        "go" => "go",
        "c" => "c",
        "cpp" => "cpp",
        "h" => "cpp",
        "rb" => "ruby",
        "swift" => "swift",
        "cs" => "csharp",
        "toml" => "toml",
        "lua" => "lua",
        _ => return None
    })
}
