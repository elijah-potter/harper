use std::path::Path;

use harper_comments::TreeSitterParser;
use harper_core::{Document, FullDictionary, LintGroup, LintGroupConfig, Linter};

/// Creates a unit test checking that the linting of a source file in
/// `language_support_sources` produces the expected number of lints.
macro_rules! create_test {
    ($filename:ident.$ext:ident, $correct_expected:expr) => {
        paste::paste! {
            #[test]
            fn [<lints_$ext _ $filename _correctly>](){
                 let filename = concat!(stringify!($filename), ".", stringify!($ext));
                 let source = include_str!(
                    concat!(
                        "./language_support_sources/",
                        concat!(
                        stringify!($filename), ".", stringify!($ext))
                    )
                 );

                 let parser = TreeSitterParser::new_from_filename(Path::new(filename)).unwrap();
                 let document = Document::new(&source, Box::new(parser));

                 let mut linter = LintGroup::new(
                     LintGroupConfig::default(),
                     FullDictionary::create_from_curated()
                 );
                 let lints = linter.lint(&document);

                 dbg!(&lints);
                 assert_eq!(lints.len(), $correct_expected);
            }
        }
    };
}

create_test!(multiline_comments.cpp, 3);
create_test!(multiline_comments.ts, 3);
create_test!(clean.rs, 0);
create_test!(jsdoc.ts, 4);
create_test!(issue_96.lua, 0);
