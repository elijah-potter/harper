use harper_core::Document;
use harper_gestalt::GestaltParser;
use harper_linting::{LintGroup, LintGroupConfig, Linter};
use harper_spell::FstDictionary;
use std::path::Path;

/// Creates a unit test checking that the linting of a source file in
/// `tests_sources` produces the expected number of lints.
macro_rules! create_test {
    ($filename:ident.$ext:ident, $correct_expected:expr) => {
        paste::paste! {
            #[test]
            fn [<lints_$ext _ $filename _correctly>](){
                 let filename = concat!(stringify!($filename), ".", stringify!($ext));
                 let source = include_str!(
                    concat!(
                        "./test_sources/",
                        concat!(
                        stringify!($filename), ".", stringify!($ext))
                    )
                 );

                 let mut parser = GestaltParser::new_from_filename(Path::new(filename)).unwrap();
                 let dict = FstDictionary::curated();
                 let document = Document::new(&source, &mut parser, &dict);

                 let mut linter = LintGroup::new(
                     LintGroupConfig::default(),
                     dict
                 );
                 let lints = linter.lint(&document);

                 dbg!(&lints);
                 assert_eq!(lints.len(), $correct_expected);

                 // Make sure that all generated tokens span real characters
                 for token in document.tokens(){
                     assert!(token.span.try_get_content(document.get_source()).is_some());
                 }
            }
        }
    };
}

create_test!(whack_bullets.md, 1);
create_test!(preexisting.md, 0);
create_test!(issue_109.md, 0);
create_test!(issue_109_ext.md, 0);
create_test!(chinese_lorem_ipsum.md, 2);
create_test!(obsidian_links.md, 2);

create_test!(multiline_comments.cpp, 3);
create_test!(multiline_comments.ts, 3);
create_test!(clean.rs, 0);
create_test!(jsdoc.ts, 4);
create_test!(issue_96.lua, 0);
create_test!(merged_lines.ts, 1);
create_test!(javadoc_clean_simple.java, 0);
create_test!(javadoc_complex.java, 4);
create_test!(issue_132.rs, 1);

// These are to make sure nothing crashes.
create_test!(empty.js, 0);
create_test!(issue_229.js, 0);
create_test!(issue_229.c, 0);
create_test!(issue_229.cs, 0);
create_test!(eof.rs, 0);

create_test!(run_on.html, 0);
