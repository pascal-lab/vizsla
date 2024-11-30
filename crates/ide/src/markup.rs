use std::fmt;

#[derive(Clone, Default, Debug, Hash, PartialEq, Eq)]
pub struct Markup {
    text: String,
}

impl From<Markup> for String {
    fn from(markup: Markup) -> Self {
        markup.text
    }
}

impl From<String> for Markup {
    fn from(text: String) -> Self {
        Markup { text }
    }
}

impl fmt::Display for Markup {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.text, f)
    }
}

impl Markup {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, contents: &str) {
        self.text.push_str(contents);
    }

    pub fn as_str(&self) -> &str {
        self.text.as_str()
    }

    pub fn line_break(&mut self) {
        if !self.text.is_empty() {
            self.text.push_str("\n------------------\n");
        }
    }

    pub fn push_with_plain_fence(&mut self, contents: &str) {
        if !self.text.is_empty() {
            self.line_break();
        }
        self.text.push_str("```\n");
        self.text.push_str(contents);
        self.text.push_str("\n```\n");
    }
}
