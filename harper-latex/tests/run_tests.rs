use harper_core::{Document, FullDictionary, LintGroup, LintGroupConfig, Linter};

/// Creates a unit test checking that the linting of a LaTeX document (in
/// `tests_sources`) produces the expected number of lints.
macro_rules! create_test {
    ($filename:ident.tex, $correct_expected:expr) => {
        paste::paste! {
            #[ignore]
            #[test]
            fn [<lints_ $filename _correctly>](){
                 let source = include_str!(
                    concat!(
                        "./test_sources/",
                        concat!(stringify!($filename), ".tex")
                    )
                 );

                 let document = Document::new_markdown(&source);

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

create_test!(complex_example_document.tex, 0);
create_test!(small_example_document.tex, 0);
