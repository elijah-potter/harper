/// This is an example of an problematic comment.
/// It should produce one error.
int test() {}

/***
 * This is an example of a possible error:
 * these subsequent lines should not be considered a new sentence and should
 * produce no errors.
 */
int arbitrary() {}

/// Let's aadd a cuple spelling errors for good measure.
