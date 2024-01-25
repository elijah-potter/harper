use harper_core::Span;
use tree_sitter::{Parser, TreeCursor};

/// Extract each comment astris a seperate block.
pub fn extract_comments_rust(text: &str) -> Vec<Span> {
    let mut parser = Parser::new();
    parser.set_language(tree_sitter_rust::language()).unwrap();

    // TODO: Use incremental parsing
    let Some(root) = parser.parse(text, None) else {
        return vec![];
    };

    let mut comments = Vec::new();

    extract_comments(&mut root.walk(), &mut comments);

    comments
}

fn extract_comments(cursor: &mut TreeCursor, comments: &mut Vec<Span>) {
    if !cursor.goto_first_child() {
        return;
    }

    while cursor.goto_next_sibling() {
        let node = cursor.node();

        if node.kind().contains("comment") {
            comments.push(node.byte_range().into());
        }

        extract_comments(cursor, comments);
    }

    cursor.goto_parent();
}
