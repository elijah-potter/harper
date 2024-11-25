use itertools::Itertools;

use typst_syntax::ast::{AstNode, Expr, Markup};

use super::{Parser, PlainEnglish};
use crate::{parsers::StrParser, Token, TokenKind, WordMetadata};

/// A parser that wraps the [`PlainEnglish`] parser that allows one to parse
/// Typst files.
pub struct Typst;

macro_rules! constant_token {
    ($offset:ident, $doc:ident, $a:ident, $to:expr) => {{
        let range = $doc.range($a.span()).unwrap();
        *$offset += range.len();
        Some(vec![Token {
            span: range.into(),
            kind: $to,
        }])
    }};
}

fn recursive_env(
    exprs: &mut dyn Iterator<Item = typst_syntax::ast::Expr>,
    doc: &typst_syntax::Source,
    parser: &mut PlainEnglish,
    offset: &mut usize,
) -> Option<Vec<Token>> {
    Some(
        exprs
            .filter_map(|e| map_token(e, doc, parser, offset))
            .flatten()
            .collect_vec(),
    )
}

fn map_token(
    ex: typst_syntax::ast::Expr,
    doc: &typst_syntax::Source,
    parser: &mut PlainEnglish,
    offset: &mut usize,
) -> Option<Vec<Token>> {
    match ex {
        Expr::Text(text) => Some(
            parser
                .parse_str(text.get())
                .into_iter()
                .map(|mut t| {
                    t.span.push_by(*offset);
                    t
                })
                .collect_vec(),
        ),
        Expr::Space(a) => constant_token!(offset, doc, a, TokenKind::Space(1)),
        Expr::Linebreak(a) => constant_token!(offset, doc, a, TokenKind::Newline(1)),
        Expr::Parbreak(a) => constant_token!(offset, doc, a, TokenKind::Newline(2)),
        Expr::Escape(_) => None,
        Expr::Shorthand(_) => None,
        Expr::SmartQuote(_) => None,
        Expr::Strong(strong) => recursive_env(&mut strong.body().exprs(), doc, parser, offset),
        Expr::Emph(emph) => recursive_env(&mut emph.body().exprs(), doc, parser, offset),
        Expr::Raw(_) => None,
        Expr::Link(a) => constant_token!(offset, doc, a, TokenKind::Url),
        Expr::Label(label) => Some(
            parser
                .parse_str(label.get())
                .into_iter()
                .map(|mut t| {
                    t.span.push_by(*offset);
                    t
                })
                .collect_vec(),
        ),
        Expr::Ref(a) => {
            constant_token!(offset, doc, a, TokenKind::Word(WordMetadata::default()))
        }
        Expr::Heading(heading) => recursive_env(&mut heading.body().exprs(), doc, parser, offset),
        Expr::List(list_item) => recursive_env(&mut list_item.body().exprs(), doc, parser, offset),
        Expr::Enum(enum_item) => recursive_env(&mut enum_item.body().exprs(), doc, parser, offset),
        Expr::Term(term_item) => recursive_env(
            &mut term_item
                .term()
                .exprs()
                .chain(term_item.description().exprs()),
            doc,
            parser,
            offset,
        ),
        Expr::Equation(a) => constant_token!(offset, doc, a, TokenKind::Unlintable),
        Expr::Math(_) => None,
        Expr::MathIdent(_) => None,
        Expr::MathShorthand(_) => None,
        Expr::MathAlignPoint(_) => None,
        Expr::MathDelimited(_) => None,
        Expr::MathAttach(_) => None,
        Expr::MathPrimes(_) => None,
        Expr::MathFrac(_) => None,
        Expr::MathRoot(_) => None,
        Expr::Ident(a) => constant_token!(offset, doc, a, TokenKind::Word(WordMetadata::default())),
        Expr::None(a) => constant_token!(offset, doc, a, TokenKind::Word(WordMetadata::default())),
        Expr::Auto(a) => constant_token!(offset, doc, a, TokenKind::Word(WordMetadata::default())),
        Expr::Bool(a) => constant_token!(offset, doc, a, TokenKind::Word(WordMetadata::default())),
        Expr::Int(int) => todo!(),
        Expr::Float(float) => todo!(),
        Expr::Numeric(a) => constant_token!(offset, doc, a, TokenKind::Unlintable),
        Expr::Str(text) => Some(
            parser
                .parse_str(text.get())
                .into_iter()
                .map(|mut t| {
                    t.span.push_by(*offset);
                    t
                })
                .collect_vec(),
        ),
        Expr::Code(a) => constant_token!(offset, doc, a, TokenKind::Unlintable),
        Expr::Content(content_block) => {
            recursive_env(&mut content_block.body().exprs(), doc, parser, offset)
        }
        Expr::Parenthesized(parenthesized) => map_token(parenthesized.expr(), doc, parser, offset),
        Expr::Array(array) => Some(
            array
                .items()
                .filter_map(|i| {
                    if let typst_syntax::ast::ArrayItem::Pos(e) = i {
                        map_token(e, doc, parser, offset)
                    } else {
                        None
                    }
                })
                .flatten()
                .collect_vec(),
        ),
        Expr::Dict(dict) => todo!(),
        Expr::Unary(unary) => todo!(),
        Expr::Binary(binary) => todo!(),
        Expr::FieldAccess(field_access) => todo!(),
        Expr::FuncCall(func_call) => todo!(),
        Expr::Closure(closure) => todo!(),
        Expr::Let(let_binding) => todo!(),
        Expr::DestructAssign(destruct_assignment) => todo!(),
        Expr::Set(set_rule) => todo!(),
        Expr::Show(show_rule) => todo!(),
        Expr::Contextual(contextual) => todo!(),
        Expr::Conditional(conditional) => todo!(),
        Expr::While(while_loop) => todo!(),
        Expr::For(for_loop) => todo!(),
        Expr::Import(module_import) => todo!(),
        Expr::Include(module_include) => todo!(),
        Expr::Break(loop_break) => todo!(),
        Expr::Continue(loop_continue) => todo!(),
        Expr::Return(func_return) => todo!(),
    }
}

impl Parser for Typst {
    fn parse(&mut self, source: &[char]) -> Vec<Token> {
        let mut english_parser = PlainEnglish;

        let source_str: String = source.iter().collect();
        let typst_document = typst_syntax::Source::detached(source_str);
        let typst_tree = Markup::from_untyped(typst_document.root())
            .expect("Unable to create typst document from parsed tree!");
        let mut offset = 0;

        // NOTE: the range spits out __byte__ indices, not char indices.
        // This is why we keep track above.
        typst_tree
            .exprs()
            .filter_map(|ex| map_token(ex, &typst_document, &mut english_parser, &mut offset))
            .flatten()
            .collect_vec()
    }
}
