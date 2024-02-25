use std::collections::HashMap;

use harper_core::{Lint, Suggestion};
use tower_lsp::lsp_types::{
    CodeAction,
    CodeActionKind,
    CodeActionOrCommand,
    Command,
    Diagnostic,
    TextEdit,
    Url,
    WorkspaceEdit
};

use crate::pos_conv::span_to_range;

pub fn lints_to_diagnostics(source: &[char], lints: &[Lint]) -> Vec<Diagnostic> {
    lints
        .iter()
        .map(|lint| lint_to_diagnostic(lint, source))
        .collect()
}

pub fn lint_to_code_actions<'a>(
    lint: &'a Lint,
    url: &'a Url,
    source: &'a [char]
) -> Vec<CodeActionOrCommand> {
    let mut results = Vec::new();

    results.extend(
        lint.suggestions
            .iter()
            .flat_map(|suggestion| {
                let range = span_to_range(source, lint.span);

                let Suggestion::ReplaceWith(with) = suggestion;
                Some(CodeAction {
                    title: suggestion.to_string(),
                    kind: Some(CodeActionKind::QUICKFIX),
                    diagnostics: None,
                    edit: Some(WorkspaceEdit {
                        changes: Some(HashMap::from([(
                            url.clone(),
                            vec![TextEdit {
                                range,

                                new_text: with.iter().collect()
                            }]
                        )])),
                        document_changes: None,
                        change_annotations: None
                    }),
                    command: None,
                    is_preferred: None,
                    disabled: None,
                    data: None
                })
            })
            .map(CodeActionOrCommand::CodeAction)
    );

    if lint.lint_kind.is_spelling() {
        let orig = lint.span.get_content_string(source);

        results.push(CodeActionOrCommand::Command(Command::new(
            format!("Add \"{}\" to the global dictionary.", orig),
            "AddToUserDict".to_string(),
            Some(vec![orig.clone().into()])
        )));

        results.push(CodeActionOrCommand::Command(Command::new(
            format!("Add \"{}\" to the file dictionary.", orig),
            "AddToFileDict".to_string(),
            Some(vec![orig.into(), url.to_string().into()])
        )))
    }

    results
}

fn lint_to_diagnostic(lint: &Lint, source: &[char]) -> Diagnostic {
    let range = span_to_range(source, lint.span);

    Diagnostic {
        range,
        severity: None,
        code: None,
        code_description: None,
        source: Some("Harper".to_string()),
        message: lint.message.clone(),
        related_information: None,
        tags: None,
        data: None
    }
}
