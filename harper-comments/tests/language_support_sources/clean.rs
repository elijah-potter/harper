/// This is an example Rust file that should produce no Harper lints.

struct TestStruct {}

impl TestStruct {
    /// This is a test function.
    /// It has a [link](https://example.com) embedded inside
    fn test_function() {}

    /// This is another test function.
    /// It has another [link](https://example.com) embedded inside
    fn test_function() {}

    /// This is some gibberish to try to trigger a lint for sentences that continue for too long
    ///
    /// This is some gibberish to try to trigger a lint for sentences that continue for too long
    ///
    /// This is some gibberish to try to trigger a lint for sentences that continue for too long
    /// 
    /// This is some gibberish to try to trigger a lint for sentences that continue for too long
    ///
    /// This is some gibberish to try to trigger a lint for sentences that continue for too long
}

