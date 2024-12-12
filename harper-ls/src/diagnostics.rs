use std::collections::HashMap;

use harper_core::linting::{Lint, LintSeverity, Suggestion};
use tower_lsp::lsp_types::{
    CodeAction, CodeActionKind, CodeActionOrCommand, Command, Diagnostic, TextEdit, Url,
    WorkspaceEdit,
};

use crate::config::{severity_to_lsp, CodeActionConfig};
use crate::pos_conv::span_to_range;

pub fn lints_to_diagnostics(
    source: &[char],
    lints: &[Lint],
    default_severity: LintSeverity,
) -> Vec<Diagnostic> {
    lints
        .iter()
        .map(|lint| lint_to_diagnostic(lint, source, default_severity))
        .collect()
}

pub fn lint_to_code_actions<'a>(
    lint: &'a Lint,
    url: &'a Url,
    source: &'a [char],
    config: &CodeActionConfig,
) -> Vec<CodeActionOrCommand> {
    let mut results = Vec::new();

    results.extend(
        lint.suggestions
            .iter()
            .flat_map(|suggestion| {
                let range = span_to_range(source, lint.span);

                let replace_string = match suggestion {
                    Suggestion::ReplaceWith(with) => with.iter().collect(),
                    Suggestion::Remove => "".to_string(),
                };

                Some(CodeAction {
                    title: suggestion.to_string(),
                    kind: Some(CodeActionKind::QUICKFIX),
                    diagnostics: None,
                    edit: Some(WorkspaceEdit {
                        changes: Some(HashMap::from([(
                            url.clone(),
                            vec![TextEdit {
                                range,
                                new_text: replace_string,
                            }],
                        )])),
                        document_changes: None,
                        change_annotations: None,
                    }),
                    command: None,
                    is_preferred: None,
                    disabled: None,
                    data: None,
                })
            })
            .map(CodeActionOrCommand::CodeAction),
    );

    if lint.lint_kind.is_spelling() {
        let orig = lint.span.get_content_string(source);

        results.push(CodeActionOrCommand::Command(Command::new(
            format!("Add \"{}\" to the global dictionary.", orig),
            "HarperAddToUserDict".to_string(),
            Some(vec![orig.clone().into(), url.to_string().into()]),
        )));

        results.push(CodeActionOrCommand::Command(Command::new(
            format!("Add \"{}\" to the file dictionary.", orig),
            "HarperAddToFileDict".to_string(),
            Some(vec![orig.into(), url.to_string().into()]),
        )));

        if config.force_stable {
            results.reverse();
        }
    }

    results
}

fn lint_to_diagnostic(lint: &Lint, source: &[char], default_severity: LintSeverity) -> Diagnostic {
    let range = span_to_range(source, lint.span);

    Diagnostic {
        range,
        severity: lint
            .severity
            .or(Some(default_severity))
            .map(severity_to_lsp),
        code: None,
        code_description: None,
        source: Some("Harper".to_string()),
        message: lint.message.clone(),
        related_information: None,
        tags: None,
        data: None,
    }
}
