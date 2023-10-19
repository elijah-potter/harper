/// Represents a position in a given text document
pub struct LineCol {
    pub line: usize,
    pub col: usize,
}

impl LineCol {
    pub fn new(line: usize, col: usize) -> Self {
        Self { line, col }
    }

    pub fn from_index(index: usize, source: &[char]) -> LineCol {
        let mut lines = 0;
        let mut last_newline_index = 0;

        for (index, c) in source[0..index].iter().enumerate() {
            if *c == '\n' {
                lines += 1;
                last_newline_index = index;
            }
        }

        LineCol {
            line: lines,
            col: index - last_newline_index - 1,
        }
    }
}
