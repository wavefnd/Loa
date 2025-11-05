#[derive(Debug)]
pub enum LoaErrorKind {
    UnexpectedToken(String),
    ExpectedToken(String),
    UnexpectedChar(char),
    SyntaxError(String),
}

#[derive(Debug)]
pub struct LoaError {
    pub kind: LoaErrorKind,
    pub message: String,
    pub file: String,
    pub line: usize,
    pub column: usize,
    pub source: Option<String>,
    pub label: Option<String>,
}

impl LoaError {
    pub fn new(kind: LoaErrorKind, message: impl Into<String>, file: impl Into<String>, line: usize, column: usize) -> Self {
        Self {
            kind,
            message: message.into(),
            file: file.into(),
            line,
            column,
            source: None,
            label: None,
        }
    }

    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        self.source = Some(source.into());
        self
    }

    pub fn with_label(mut self, label: impl Into<String>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn display(&self) {
        eprintln!("error: {}", self.message);
        eprintln!("  --> {}:{}:{}", self.file, self.line, self.column);
        eprintln!("   |");

        if let Some(source_line) = &self.source {
            eprintln!("{:>3} | {}", self.line, source_line);
            let arrow_line = format!("{:>3} | {:>width$}^", "", "", width = self.column - 1);
            if let Some(label) = &self.label {
                eprintln!("   | {} {}", &arrow_line[6..], label);
            } else {
                eprintln!("   | {}", &arrow_line[6..]);
            }
        } else {
            eprintln!("   | (source unavailable)");
        }
    }
}