use crate::span::Span;

pub struct Lint {
    span: Span,
    lint_kind: LintKind,
}

pub enum LintKind {}
