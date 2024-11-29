use itertools::Itertools;
use std::collections::VecDeque;
use typst_syntax::ast::{AstNode, Expr, Markup};

use super::{Parser, PlainEnglish};
use crate::{
    parsers::StrParser,
    patterns::{PatternExt, SequencePattern},
    ConjunctionData, Lrc, NounData, Punctuation, Span, Token, TokenKind, VecExt, WordMetadata,
};

/// A parser that wraps the [`PlainEnglish`] parser that allows one to parse
/// Typst files.
pub struct Typst;

macro_rules! constant_token {
    ($doc:ident, $a:expr, $to:expr) => {{
        Some(vec![Token {
            span: $doc.range($a.span()).unwrap().into(),
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
) -> Option<Vec<Token>> {
    Some(
        exprs
            .filter_map(|e| map_token(e, doc, parser))
            .flatten()
            .collect_vec(),
    )
}

fn parse_english(
    str: impl Into<String>,
    doc: &typst_syntax::Source,
    parser: &mut PlainEnglish,
    span: &typst_syntax::Span,
) -> Option<Vec<Token>> {
    let res = parser
        .parse_str(str.into())
        .into_iter()
        .map(|mut t| {
            t.span.push_by(doc.range(*span).unwrap().start);
            t
        })
        .collect_vec();
    Some(res)
}

fn parse_dict(
    dict: &mut dyn Iterator<Item = typst_syntax::ast::DictItem>,
    doc: &typst_syntax::Source,
    parser: &mut PlainEnglish,
) -> Option<Vec<Token>> {
    Some(
        dict.filter_map(|di| match di {
            typst_syntax::ast::DictItem::Named(named) => merge_expr!(
                constant_token!(doc, named.name(), TokenKind::Word(WordMetadata::default())),
                map_token(named.expr(), doc, parser),
                parse_pattern(named.pattern(), doc, parser)
            ),
            typst_syntax::ast::DictItem::Keyed(keyed) => merge_expr!(
                map_token(keyed.key(), doc, parser),
                map_token(keyed.expr(), doc, parser)
            ),
            typst_syntax::ast::DictItem::Spread(spread) => spread.sink_ident().map_or_else(
                || {
                    spread
                        .sink_expr()
                        .and_then(|expr| map_token(expr, doc, parser))
                },
                |ident| constant_token!(doc, ident, TokenKind::Word(WordMetadata::default())),
            ),
        })
        .flatten()
        .collect(),
    )
}

fn parse_pattern(
    pat: typst_syntax::ast::Pattern,
    doc: &typst_syntax::Source,
    parser: &mut PlainEnglish,
) -> Option<Vec<Token>> {
    match pat {
        typst_syntax::ast::Pattern::Normal(expr) => map_token(expr, doc, parser),
        typst_syntax::ast::Pattern::Placeholder(underscore) => {
            constant_token!(doc, underscore, TokenKind::Unlintable)
        }
        typst_syntax::ast::Pattern::Parenthesized(parenthesized) => merge_expr!(
            map_token(parenthesized.expr(), doc, parser),
            parse_pattern(parenthesized.pattern(), doc, parser)
        ),
        typst_syntax::ast::Pattern::Destructuring(destructuring) => Some(
            destructuring
                .items()
                .filter_map(|item| match item {
                    typst_syntax::ast::DestructuringItem::Pattern(pattern) => {
                        parse_pattern(pattern, doc, parser)
                    }
                    typst_syntax::ast::DestructuringItem::Named(named) => merge_expr!(
                        constant_token!(
                            doc,
                            named.name(),
                            TokenKind::Word(WordMetadata::default())
                        ),
                        parse_pattern(named.pattern(), doc, parser)
                    ),
                    typst_syntax::ast::DestructuringItem::Spread(spread) => {
                        spread.sink_ident().map_or_else(
                            || {
                                spread
                                    .sink_expr()
                                    .and_then(|expr| map_token(expr, doc, parser))
                            },
                            |ident| {
                                constant_token!(
                                    doc,
                                    ident,
                                    TokenKind::Word(WordMetadata::default())
                                )
                            },
                        )
                    }
                })
                .flatten()
                .collect(),
        ),
    }
}

fn map_token(
    ex: typst_syntax::ast::Expr,
    doc: &typst_syntax::Source,
    parser: &mut PlainEnglish,
) -> Option<Vec<Token>> {
    match ex {
        Expr::Text(text) => parse_english(text.get(), doc, parser, &text.span()),
        Expr::Space(a) => constant_token!(doc, a, TokenKind::Space(1)),
        Expr::Linebreak(a) => constant_token!(doc, a, TokenKind::Newline(1)),
        Expr::Parbreak(a) => constant_token!(doc, a, TokenKind::ParagraphBreak),
        Expr::Escape(a) => constant_token!(doc, a, TokenKind::Unlintable),
        Expr::Shorthand(a) => constant_token!(doc, a, TokenKind::Unlintable),
        Expr::SmartQuote(quote) => {
            if quote.double() {
                constant_token!(
                    doc,
                    quote,
                    TokenKind::Punctuation(Punctuation::Quote(crate::Quote { twin_loc: None }))
                )
            } else {
                constant_token!(doc, quote, TokenKind::Punctuation(Punctuation::Apostrophe))
            }
        }
        Expr::Strong(strong) => recursive_env(&mut strong.body().exprs(), doc, parser),
        Expr::Emph(emph) => recursive_env(&mut emph.body().exprs(), doc, parser),
        Expr::Raw(a) => constant_token!(doc, a, TokenKind::Unlintable),
        Expr::Link(a) => constant_token!(doc, a, TokenKind::Url),
        Expr::Label(label) => parse_english(label.get(), doc, parser, &label.span()),
        Expr::Ref(a) => {
            constant_token!(doc, a, TokenKind::Word(WordMetadata::default()))
        }
        Expr::Heading(heading) => recursive_env(&mut heading.body().exprs(), doc, parser),
        Expr::List(list_item) => recursive_env(&mut list_item.body().exprs(), doc, parser),
        Expr::Enum(enum_item) => recursive_env(&mut enum_item.body().exprs(), doc, parser),
        Expr::Term(term_item) => recursive_env(
            &mut term_item
                .term()
                .exprs()
                .chain(term_item.description().exprs()),
            doc,
            parser,
        ),
        Expr::Equation(a) => constant_token!(doc, a, TokenKind::Unlintable),
        Expr::Math(_) => panic!("Unexpected math outside equation environment."),
        Expr::MathIdent(_) => panic!("Unexpected math outside equation environment."),
        Expr::MathShorthand(_) => panic!("Unexpected math outside equation environment."),
        Expr::MathAlignPoint(_) => panic!("Unexpected math outside equation environment."),
        Expr::MathDelimited(_) => panic!("Unexpected math outside equation environment."),
        Expr::MathAttach(_) => panic!("Unexpected math outside equation environment."),
        Expr::MathPrimes(_) => panic!("Unexpected math outside equation environment."),
        Expr::MathFrac(_) => panic!("Unexpected math outside equation environment."),
        Expr::MathRoot(_) => panic!("Unexpected math outside equation environment."),
        Expr::Ident(a) => constant_token!(doc, a, TokenKind::Word(WordMetadata::default())),
        Expr::None(a) => constant_token!(doc, a, TokenKind::Word(WordMetadata::default())),
        Expr::Auto(a) => constant_token!(doc, a, TokenKind::Word(WordMetadata::default())),
        Expr::Bool(a) => constant_token!(doc, a, TokenKind::Word(WordMetadata::default())),
        Expr::Int(int) => {
            constant_token!(doc, int, TokenKind::Number((int.get() as f64).into(), None))
        }
        Expr::Float(float) => {
            constant_token!(doc, float, TokenKind::Number(float.get().into(), None))
        }
        Expr::Numeric(a) => constant_token!(doc, a, TokenKind::Unlintable),
        Expr::Str(text) => parse_english(text.get(), doc, parser, &text.span()),
        Expr::Code(a) => constant_token!(doc, a, TokenKind::Unlintable),
        Expr::Content(content_block) => {
            recursive_env(&mut content_block.body().exprs(), doc, parser)
        }
        Expr::Parenthesized(parenthesized) => map_token(parenthesized.expr(), doc, parser),
        Expr::Array(array) => Some(
            array
                .items()
                .filter_map(|i| {
                    if let typst_syntax::ast::ArrayItem::Pos(e) = i {
                        map_token(e, doc, parser)
                    } else {
                        None
                    }
                })
                .flatten()
                .collect_vec(),
        ),
        Expr::Dict(a) => parse_dict(&mut a.items(), doc, parser),
        Expr::Unary(a) => constant_token!(doc, a, TokenKind::Unlintable),
        Expr::Binary(a) => constant_token!(doc, a, TokenKind::Unlintable),
        Expr::FieldAccess(field_access) => merge_expr!(
            map_token(field_access.target(), doc, parser),
            constant_token!(
                doc,
                field_access.field(),
                TokenKind::Word(WordMetadata::default())
            )
        ),
        Expr::FuncCall(a) => constant_token!(doc, a, TokenKind::Unlintable),
        Expr::Closure(a) => constant_token!(doc, a, TokenKind::Unlintable),
        Expr::Let(let_binding) => let_binding.init().and_then(|e| map_token(e, doc, parser)),
        Expr::DestructAssign(destruct_assignment) => {
            map_token(destruct_assignment.value(), doc, parser)
        }
        Expr::Set(set_rule) => merge_expr!(
            map_token(set_rule.target(), doc, parser),
            map_token(set_rule.condition()?, doc, parser)
        ),
        Expr::Show(show_rule) => merge_expr!(
            map_token(show_rule.transform(), doc, parser),
            map_token(show_rule.selector()?, doc, parser)
        ),
        Expr::Contextual(contextual) => map_token(contextual.body(), doc, parser),
        Expr::Conditional(conditional) => merge_expr!(
            map_token(conditional.condition(), doc, parser),
            map_token(conditional.if_body(), doc, parser),
            map_token(conditional.else_body()?, doc, parser)
        ),
        Expr::While(while_loop) => merge_expr!(
            map_token(while_loop.condition(), doc, parser),
            map_token(while_loop.body(), doc, parser)
        ),
        Expr::For(for_loop) => merge_expr!(
            map_token(for_loop.iterable(), doc, parser),
            map_token(for_loop.body(), doc, parser)
        ),
        Expr::Import(module_import) => {
            merge_expr!(
                map_token(module_import.source(), doc, parser),
                constant_token!(
                    doc,
                    module_import.new_name()?,
                    TokenKind::Word(WordMetadata::default())
                )
            )
        }
        Expr::Include(module_include) => map_token(module_include.source(), doc, parser),
        Expr::Break(a) => constant_token!(doc, a, TokenKind::Unlintable),
        Expr::Continue(a) => constant_token!(doc, a, TokenKind::Unlintable),
        Expr::Return(a) => constant_token!(doc, a, TokenKind::Unlintable),
    }
}

thread_local! {
    static WORD_APOSTROPHE_WORD: Lrc<SequencePattern> = Lrc::new(SequencePattern::default()
                .then_any_word()
                .then_apostrophe()
                .then_any_word());
}

impl Parser for Typst {
    fn parse(&mut self, source: &[char]) -> Vec<Token> {
        let mut english_parser = PlainEnglish;

        let source_str: String = source.iter().collect();
        let typst_document = typst_syntax::Source::detached(source_str);
        let typst_tree = Markup::from_untyped(typst_document.root())
            .expect("Unable to create typst document from parsed tree!");

        // NOTE: the range spits out __byte__ indices, not char indices.
        // This is why we keep track above.
        let mut tokens = typst_tree
            .exprs()
            .filter_map(|ex| map_token(ex, &typst_document, &mut english_parser))
            .flatten()
            .collect_vec();

        // Consolidate conjunctions
        let mut to_remove = VecDeque::default();
        for tok_span in WORD_APOSTROPHE_WORD
            .with(|v| v.clone())
            .find_all_matches(&tokens, source)
        {
            let start_tok = &tokens[tok_span.start];
            let end_tok = &tokens[tok_span.end - 1];
            let char_span = Span::new(start_tok.span.start, end_tok.span.end);

            if let TokenKind::Word(metadata) = start_tok.kind {
                tokens[tok_span.start].kind =
                    TokenKind::Word(if end_tok.span.get_content(source) == ['s'] {
                        WordMetadata {
                            noun: Some(NounData {
                                is_possessive: Some(true),
                                ..metadata.noun.unwrap_or_default()
                            }),
                            conjunction: None,
                            ..metadata
                        }
                    } else {
                        WordMetadata {
                            noun: metadata.noun.map(|noun| NounData {
                                is_possessive: Some(false),
                                ..noun
                            }),
                            conjunction: Some(ConjunctionData {}),
                            ..metadata
                        }
                    });

                tokens[tok_span.start].span = char_span;
                to_remove.extend(tok_span.start + 1..tok_span.end);
            } else {
                panic!("Apostrophe consolidation does not start with Word Token!")
            }
        }
        tokens.remove_indices(to_remove.into_iter().sorted().unique().collect());

        tokens
    }
}

#[cfg(test)]
mod tests {
    use ordered_float::OrderedFloat;

    use super::Typst;
    use crate::{parsers::StrParser, NounData, Punctuation, TokenKind, WordMetadata};

    #[test]
    fn conjunction() {
        let source = r"doesn't";

        let tokens = Typst.parse_str(source);

        let token_kinds = tokens.iter().map(|t| t.kind).collect::<Vec<_>>();

        dbg!(&token_kinds);

        assert_eq!(token_kinds.len(), 1);
        assert!(token_kinds.into_iter().all(|t| t.is_conjunction()))
    }

    #[test]
    fn possessive() {
        let source = r"person's";

        let tokens = Typst.parse_str(source);

        let token_kinds = tokens.iter().map(|t| t.kind).collect::<Vec<_>>();

        dbg!(&token_kinds);

        assert_eq!(token_kinds.len(), 1);
        assert!(token_kinds.into_iter().all(|t| {
            matches!(
                t,
                TokenKind::Word(WordMetadata {
                    noun: Some(NounData {
                        is_possessive: Some(true),
                        ..
                    }),
                    ..
                })
            )
        }))
    }

    #[test]
    fn number() {
        let source = r"12 is larger than 11, but much less than 11!";

        let tokens = Typst.parse_str(source);

        let token_kinds = tokens.iter().map(|t| t.kind).collect::<Vec<_>>();

        dbg!(&token_kinds);

        assert!(matches!(
            token_kinds.as_slice(),
            &[
                TokenKind::Number(OrderedFloat(12.0), None),
                TokenKind::Space(1),
                TokenKind::Word(_),
                TokenKind::Space(1),
                TokenKind::Word(_),
                TokenKind::Space(1),
                TokenKind::Word(_),
                TokenKind::Space(1),
                TokenKind::Number(OrderedFloat(11.0), None),
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
                TokenKind::Number(OrderedFloat(11.0), None),
                TokenKind::Punctuation(Punctuation::Bang),
            ]
        ))
    }

    #[test]
    fn math_unlintable() {
        let source = r"$12 > 11$, $12 << 11!$";

        let tokens = Typst.parse_str(source);

        let token_kinds = tokens.iter().map(|t| t.kind).collect::<Vec<_>>();

        dbg!(&token_kinds);

        assert!(matches!(
            token_kinds.as_slice(),
            &[
                TokenKind::Unlintable,
                TokenKind::Punctuation(Punctuation::Comma),
                TokenKind::Space(1),
                TokenKind::Unlintable,
            ]
        ))
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
