use harper_core::{Document, FullDictionary, LintGroup, LintGroupConfig, Linter};

/// Creates a unit test checking that the linting of a Markdown document (in
/// `tests_sources`) produces the expected number of lints.
macro_rules! create_test {
    ($filename:ident.md, $correct_expected:expr) => {
        paste::paste! {
            #[test]
            fn [<lints_ $filename _correctly>](){
                 let source = include_str!(
                    concat!(
                        "./test_sources/",
                        concat!(stringify!($filename), ".md")
                    )
                 );

                 let dict = FullDictionary::curated();
                 let document = Document::new_markdown(&source, &dict);

                 let mut linter = LintGroup::new(
                     LintGroupConfig::default(),
                     dict
                 );
                 let lints = linter.lint(&document);

                 dbg!(&lints);
                 assert_eq!(lints.len(), $correct_expected);
            }
        }
    };
}

create_test!(whack_bullets.md, 1);
create_test!(preexisting.md, 0);
create_test!(issue_109.md, 0);
create_test!(chinese_lorem_ipsum.md, 2);
