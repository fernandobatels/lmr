//! Formats the data to be presented to the user

use serde::Deserialize;

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub enum OutputFormat {
    Plain,
}

impl OutputFormat {
    pub fn title1(&self, title: &str) -> String {
        match self {
            OutputFormat::Plain => format!("\n{}\n\n", title),
        }
    }

    pub fn title2(&self, title: &str) -> String {
        match self {
            OutputFormat::Plain => format!("{}\n\n", title),
        }
    }

    pub fn simple(&self, content: &str) -> String {
        match self {
            OutputFormat::Plain => format!("{}\n", content),
        }
    }

    pub fn break_line(&self) -> String {
        match self {
            OutputFormat::Plain => format!("\n"),
        }
    }
}

impl Default for OutputFormat {
    fn default() -> Self {
        OutputFormat::Plain
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn title1() {
        assert_eq!(
            OutputFormat::Plain.title1("Title"),
            "\nTitle\n\n".to_string()
        );
    }

    #[test]
    fn title2() {
        assert_eq!(OutputFormat::Plain.title2("Title"), "Title\n\n".to_string());
    }

    #[test]
    fn simple() {
        assert_eq!(
            OutputFormat::Plain.simple("Content"),
            "Content\n".to_string()
        );
    }

    #[test]
    fn break_line() {
        assert_eq!(OutputFormat::Plain.break_line(), "\n".to_string());
    }
}
