use super::matcher;

#[derive(Debug, Clone, Copy, thiserror::Error)]
pub enum Error {
    #[error("The provided file's item count was malformed.")]
    MalformedItemCount,
    #[error("Expected affix flag to be exactly one character.")]
    MultiCharacterFlag,
    #[error("Expected affix option to be a boolean.")]
    ExpectedBoolean,
    #[error("Expected affix option to be an unsigned integer.")]
    ExpectedUnsignedInteger,
    #[error("Could not parse because we encountered the end of the line.")]
    UnexpectedEndOfLine,
    #[error("An error occured with a condition: {0}")]
    Matcher(#[from] matcher::Error)
}
