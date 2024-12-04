#![doc = include_str!("../README.md")]

use std::path::{Path, PathBuf};

use anyhow::format_err;
use ariadne::{Color, Label, Report, ReportKind, Source};
use clap::Parser;
use harper_comments::CommentParser;
use harper_core::linting::{LintGroup, LintGroupConfig, Linter};
use harper_core::parsers::Markdown;
use harper_core::{remove_overlaps, Dictionary, Document, FstDictionary};

#[derive(Debug, Parser)]
enum Args {
    /// Lint a provided document.
    Lint {
        /// The file you wish to grammar check.
        file: PathBuf,
        /// Whether to merely print out the number of errors encountered,
        /// without further details.
        #[arg(short, long)]
        count: bool,
    },
    /// Parse a provided document and print the detected symbols.
    Parse {
        /// The file you wish to parse.
        file: PathBuf,
    },
    /// Parse a provided document and show the spans of the detected tokens.
    Spans {
        /// The file you wish to display the spans.
        file: PathBuf,
    },
    /// Emit decompressed, line-separated list of words in Harper's dictionary.
    Words,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    match args {
        Args::Lint { file, count } => {
            let (doc, source) = load_file(&file)?;

            let mut linter = LintGroup::new(LintGroupConfig::default(), FstDictionary::curated());
            let mut lints = linter.lint(&doc);

            if count {
                println!("{}", lints.len());
                return Ok(());
            }

            if lints.is_empty() {
                println!("No lints found");
                return Ok(());
            }

            remove_overlaps(&mut lints);

            let primary_color = Color::Magenta;

            let filename = file
                .file_name()
                .map(|s| s.to_string_lossy().into())
                .unwrap_or("<file>".to_string());

            let mut report_builder = Report::build(ReportKind::Advice, &filename, 0);

            for lint in lints {
                report_builder = report_builder.with_label(
                    Label::new((&filename, lint.span.into()))
                        .with_message(lint.message)
                        .with_color(primary_color),
                );
            }

            let report = report_builder.finish();
            report.print((&filename, Source::from(source)))?;

            std::process::exit(1);
        }
        Args::Parse { file } => {
            let (doc, _) = load_file(&file)?;

            for token in doc.tokens() {
                let json = serde_json::to_string(&token)?;
                println!("{}", json);
            }

            Ok(())
        }
        Args::Spans { file } => {
            let (doc, source) = load_file(&file)?;

            let primary_color = Color::Blue;
            let secondary_color = Color::Magenta;
            let filename = file
                .file_name()
                .map(|s| s.to_string_lossy().into())
                .unwrap_or("<file>".to_string());

            let mut report_builder = Report::build(ReportKind::Advice, &filename, 0);
            let mut color = primary_color;
            for token in doc.tokens() {
                report_builder = report_builder.with_label(
                    Label::new((&filename, token.span.into()))
                        .with_message(format!("[{}, {})", token.span.start, token.span.end))
                        .with_color(color),
                );

                // Alternate colors so spans are clear
                color = if color == primary_color {
                    secondary_color
                } else {
                    primary_color
                };
            }

            let report = report_builder.finish();
            report.print((&filename, Source::from(source)))?;

            std::process::exit(1);
        }
        Args::Words => {
            let dict = FstDictionary::curated();

            let mut word_str = String::new();

            for word in dict.words_iter() {
                word_str.clear();
                word_str.extend(word);

                println!("{}", word_str);
            }

            Ok(())
        }
    }
}

fn load_file(file: &Path) -> anyhow::Result<(Document, String)> {
    let source = std::fs::read_to_string(file)?;

    let mut parser: Box<dyn harper_core::parsers::Parser> =
        if let Some("md") = file.extension().map(|v| v.to_str().unwrap()) {
            Box::new(Markdown)
        } else {
            Box::new(
                CommentParser::new_from_filename(file)
                    .map(Box::new)
                    .ok_or(format_err!("Could not detect language ID."))?,
            )
        };

    Ok((Document::new_curated(&source, &mut parser), source))
}
