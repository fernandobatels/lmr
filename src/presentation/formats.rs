//! Formats the data to be presented to the user

use serde::Deserialize;

#[derive(Clone, Debug, PartialEq, Deserialize)]
pub enum OutputFormat {
    Plain,
    Html,
    Markdown,
}

impl OutputFormat {
    pub fn title1(&self, title: &str) -> String {
        match self {
            OutputFormat::Plain => format!("\n{}\n\n", title),
            OutputFormat::Html => format!("<h1>{}</h1>\n", title),
            OutputFormat::Markdown => format!("\n# {}\n\n", title),
        }
    }

    pub fn title2(&self, title: &str) -> String {
        match self {
            OutputFormat::Plain => format!("{}\n\n", title),
            OutputFormat::Html => format!("<h3>{}</h3>\n", title),
            OutputFormat::Markdown => format!("## {}\n\n", title),
        }
    }

    pub fn simple(&self, content: &str) -> String {
        match self {
            OutputFormat::Plain => format!("{}\n", content),
            OutputFormat::Html => format!("{}\n", content),
            OutputFormat::Markdown => format!("{}\n", content),
        }
    }

    pub fn break_line(&self) -> String {
        match self {
            OutputFormat::Plain => format!("\n"),
            OutputFormat::Html => format!("<br>\n"),
            OutputFormat::Markdown => format!("\n"),
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
            "\nTitle\n\n".to_string(),
            OutputFormat::Plain.title1("Title")
        );
        assert_eq!(
            "<h1>Title</h1>\n".to_string(),
            OutputFormat::Html.title1("Title")
        );
        assert_eq!(
            "\n# Title\n\n".to_string(),
            OutputFormat::Markdown.title1("Title")
        );
    }

    #[test]
    fn title2() {
        assert_eq!("Title\n\n".to_string(), OutputFormat::Plain.title2("Title"));
        assert_eq!(
            "<h3>Title</h3>\n".to_string(),
            OutputFormat::Html.title2("Title")
        );
        assert_eq!(
            "## Title\n\n".to_string(),
            OutputFormat::Markdown.title2("Title")
        );
    }

    #[test]
    fn simple() {
        assert_eq!(
            "Content\n".to_string(),
            OutputFormat::Plain.simple("Content")
        );
        assert_eq!(
            "Content\n".to_string(),
            OutputFormat::Html.simple("Content")
        );
        assert_eq!(
            "Content\n".to_string(),
            OutputFormat::Markdown.simple("Content")
        );
    }

    #[test]
    fn break_line() {
        assert_eq!("\n".to_string(), OutputFormat::Plain.break_line());
        assert_eq!("<br>\n".to_string(), OutputFormat::Html.break_line());
        assert_eq!("\n".to_string(), OutputFormat::Markdown.break_line());
    }
}
