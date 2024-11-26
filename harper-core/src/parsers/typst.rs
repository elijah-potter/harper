use itertools::Itertools;

use typst_syntax::ast::{AstNode, Expr, Markup};

use super::{Parser, PlainEnglish};
use crate::{parsers::StrParser, Punctuation, Token, TokenKind, WordMetadata};

/// A parser that wraps the [`PlainEnglish`] parser that allows one to parse
/// Typst files.
pub struct Typst;

macro_rules! constant_token {
    ($offset:ident, $doc:ident, $a:expr, $to:expr) => {{
        let range = $doc.range($a.span()).unwrap();
        *$offset += range.len();
        Some(vec![Token {
            span: range.into(),
            kind: $to,
        }])
    }};
}

macro_rules! merge_expr {
    ($($inner:expr),*) => {
        Some(
            [$($inner),*]
                .into_iter()
                .flatten()
                .flatten()
                .collect_vec(),
        )
    };
}

fn recursive_env(
    exprs: &mut dyn Iterator<Item = typst_syntax::ast::Expr>,
    doc: &typst_syntax::Source,
    parser: &mut PlainEnglish,
    offset: &mut usize,
) -> Option<Vec<Token>> {
    Some(
        exprs
            .filter_map(|e| {
                let range = doc.range(e.span()).unwrap();
                *offset += range.len();
                map_token(e, doc, parser, offset)
            })
            .flatten()
            .collect_vec(),
    )
}

fn parse_english(
    str: impl Into<String>,
    parser: &mut PlainEnglish,
    offset: &mut usize,
) -> Option<Vec<Token>> {
    let res = parser
        .parse_str(str.into())
        .into_iter()
        .map(|mut t| {
            t.span.push_by(*offset);
            t
        })
        .collect_vec();
    *offset = res.last()?.span.end;
    Some(res)
}

fn map_token(
    ex: typst_syntax::ast::Expr,
    doc: &typst_syntax::Source,
    parser: &mut PlainEnglish,
    offset: &mut usize,
) -> Option<Vec<Token>> {
    match ex {
        Expr::Text(text) => parse_english(text.get(), parser, offset),
        Expr::Space(a) => constant_token!(offset, doc, a, TokenKind::Space(1)),
        Expr::Linebreak(a) => constant_token!(offset, doc, a, TokenKind::Newline(1)),
        Expr::Parbreak(a) => constant_token!(offset, doc, a, TokenKind::Newline(2)),
        Expr::Escape(a) => constant_token!(offset, doc, a, TokenKind::Unlintable),
        Expr::Shorthand(a) => constant_token!(offset, doc, a, TokenKind::Unlintable),
        Expr::SmartQuote(quote) => {
            if quote.double() {
                constant_token!(
                    offset,
                    doc,
                    quote,
                    TokenKind::Punctuation(Punctuation::Quote(crate::Quote { twin_loc: None }))
                )
            } else {
                constant_token!(
                    offset,
                    doc,
                    quote,
                    TokenKind::Punctuation(Punctuation::Apostrophe)
                )
            }
        }
        Expr::Strong(strong) => recursive_env(&mut strong.body().exprs(), doc, parser, offset),
        Expr::Emph(emph) => recursive_env(&mut emph.body().exprs(), doc, parser, offset),
        Expr::Raw(a) => constant_token!(offset, doc, a, TokenKind::Unlintable),
        Expr::Link(a) => constant_token!(offset, doc, a, TokenKind::Url),
        Expr::Label(label) => parse_english(label.get(), parser, offset),
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
        Expr::Math(_) => panic!("Unexpected math outside equation environment."),
        Expr::MathIdent(_) => panic!("Unexpected math outside equation environment."),
        Expr::MathShorthand(_) => panic!("Unexpected math outside equation environment."),
        Expr::MathAlignPoint(_) => panic!("Unexpected math outside equation environment."),
        Expr::MathDelimited(_) => panic!("Unexpected math outside equation environment."),
        Expr::MathAttach(_) => panic!("Unexpected math outside equation environment."),
        Expr::MathPrimes(_) => panic!("Unexpected math outside equation environment."),
        Expr::MathFrac(_) => panic!("Unexpected math outside equation environment."),
        Expr::MathRoot(_) => panic!("Unexpected math outside equation environment."),
        Expr::Ident(a) => constant_token!(offset, doc, a, TokenKind::Word(WordMetadata::default())),
        Expr::None(a) => constant_token!(offset, doc, a, TokenKind::Word(WordMetadata::default())),
        Expr::Auto(a) => constant_token!(offset, doc, a, TokenKind::Word(WordMetadata::default())),
        Expr::Bool(a) => constant_token!(offset, doc, a, TokenKind::Word(WordMetadata::default())),
        Expr::Int(int) => todo!(),
        Expr::Float(float) => todo!(),
        Expr::Numeric(a) => constant_token!(offset, doc, a, TokenKind::Unlintable),
        Expr::Str(text) => parse_english(text.get(), parser, offset),
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
        // TODO: actually parse dictionaries
        Expr::Dict(a) => constant_token!(offset, doc, a, TokenKind::Unlintable),
        Expr::Unary(a) => constant_token!(offset, doc, a, TokenKind::Unlintable),
        Expr::Binary(a) => constant_token!(offset, doc, a, TokenKind::Unlintable),
        Expr::FieldAccess(field_access) => merge_expr!(
            map_token(field_access.target(), doc, parser, offset),
            constant_token!(
                offset,
                doc,
                field_access.field(),
                TokenKind::Word(WordMetadata::default())
            )
        ),
        Expr::FuncCall(a) => constant_token!(offset, doc, a, TokenKind::Unlintable),
        Expr::Closure(a) => constant_token!(offset, doc, a, TokenKind::Unlintable),
        Expr::Let(let_binding) => let_binding
            .init()
            .and_then(|e| map_token(e, doc, parser, offset)),
        Expr::DestructAssign(destruct_assignment) => {
            map_token(destruct_assignment.value(), doc, parser, offset)
        }
        Expr::Set(set_rule) => merge_expr!(
            map_token(set_rule.target(), doc, parser, offset),
            map_token(set_rule.condition()?, doc, parser, offset)
        ),
        Expr::Show(show_rule) => merge_expr!(
            map_token(show_rule.transform(), doc, parser, offset),
            map_token(show_rule.selector()?, doc, parser, offset)
        ),
        Expr::Contextual(contextual) => map_token(contextual.body(), doc, parser, offset),
        Expr::Conditional(conditional) => merge_expr!(
            map_token(conditional.condition(), doc, parser, offset),
            map_token(conditional.if_body(), doc, parser, offset),
            map_token(conditional.else_body()?, doc, parser, offset)
        ),
        Expr::While(while_loop) => merge_expr!(
            map_token(while_loop.condition(), doc, parser, offset),
            map_token(while_loop.body(), doc, parser, offset)
        ),
        Expr::For(for_loop) => merge_expr!(
            map_token(for_loop.iterable(), doc, parser, offset),
            map_token(for_loop.body(), doc, parser, offset)
        ),
        Expr::Import(module_import) => {
            merge_expr!(
                map_token(module_import.source(), doc, parser, offset),
                constant_token!(
                    offset,
                    doc,
                    module_import.new_name()?,
                    TokenKind::Word(WordMetadata::default())
                )
            )
        }
        Expr::Include(module_include) => map_token(module_include.source(), doc, parser, offset),
        Expr::Break(a) => constant_token!(offset, doc, a, TokenKind::Unlintable),
        Expr::Continue(a) => constant_token!(offset, doc, a, TokenKind::Unlintable),
        Expr::Return(a) => constant_token!(offset, doc, a, TokenKind::Unlintable),
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

#[cfg(test)]
mod tests {
    use super::Typst;
    use crate::{parsers::StrParser, Punctuation, TokenKind};

    #[test]
    fn conjunction() {
        let source = r"doesn't";

        let tokens = Typst.parse_str(source);

        let token_kinds = tokens.iter().map(|t| t.kind).collect::<Vec<_>>();

        dbg!(&token_kinds);

        assert!(matches!(token_kinds.as_slice(), &[TokenKind::Word(_),]))
    }

    #[test]
    fn sentence() {
        let source = r"This is a sentence, it does not have any particularly interesting elements of the typst syntax.";

        let tokens = Typst.parse_str(source);

        let token_kinds = tokens.iter().map(|t| t.kind).collect::<Vec<_>>();

        dbg!(&token_kinds);

        assert!(matches!(
            token_kinds.as_slice(),
            &[
                TokenKind::Word(_),
                TokenKind::Space(1),
                TokenKind::Word(_),
                TokenKind::Space(1),
                TokenKind::Word(_),
                TokenKind::Space(1),
                TokenKind::Word(_),
                TokenKind::Punctuation(Punctuation::Comma),
                TokenKind::Space(1),
                TokenKind::Word(_),
                TokenKind::Space(1),
                TokenKind::Word(_),
                TokenKind::Space(1),
                TokenKind::Word(_),
                TokenKind::Space(1),
                TokenKind::Word(_),
                TokenKind::Space(1),
                TokenKind::Word(_),
                TokenKind::Space(1),
                TokenKind::Word(_),
                TokenKind::Space(1),
                TokenKind::Word(_),
                TokenKind::Space(1),
                TokenKind::Word(_),
                TokenKind::Space(1),
                TokenKind::Word(_),
                TokenKind::Space(1),
                TokenKind::Word(_),
                TokenKind::Space(1),
                TokenKind::Word(_),
                TokenKind::Space(1),
                TokenKind::Word(_),
                TokenKind::Punctuation(Punctuation::Period),
            ]
        ))
    }
}
