//! Offsets count Unicode characters, the unit of the engine's spans and of Python strings.

/// The character offset at which each line of one text starts.
#[derive(Debug, Clone)]
pub struct LineIndex {
    starts: Vec<usize>,
    length: usize,
}

impl LineIndex {
    pub fn new(text: &str) -> Self {
        let mut starts: Vec<usize> = vec![0];
        let mut length: usize = 0;
        for (index, character) in text.chars().enumerate() {
            if character == '\n' {
                starts.push(index + 1);
            }
            length = index + 1;
        }
        Self { starts, length }
    }

    /// The 1-based line and column of character `offset`, clamped to the end of the text.
    pub fn line_column(&self, offset: usize) -> (usize, usize) {
        let offset: usize = offset.min(self.length);
        let line: usize = self.starts.partition_point(|start| *start <= offset);
        (line, offset - self.starts[line - 1] + 1)
    }
}

/// A replacement of the characters `start..end` with `text`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Edit {
    pub start: usize,
    pub end: usize,
    pub text: String,
}
