use std::path::PathBuf;

use ariadne::{Color, Label, Report, ReportKind, Source};
use clap::Parser;
use harper_core::{remove_overlaps, Document, FullDictionary, LintGroup, LintGroupConfig, Linter};

#[derive(Debug, Parser)]
struct Args {
    /// The Markdown file you wish to grammar check.
    file: PathBuf
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let source = std::fs::read_to_string(&args.file)?;
    let doc = Document::new_markdown(&source);

    let mut linter = LintGroup::new(
        LintGroupConfig::default(),
        FullDictionary::create_from_curated()
    );
    let mut lints = linter.lint(&doc);

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
