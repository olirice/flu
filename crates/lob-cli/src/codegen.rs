//! Code generation for lob expressions

use crate::error::Result;
use crate::input::{InputFormat, InputSource};
use crate::output::OutputFormat;

/// Generates Rust source code from a lob expression
pub struct CodeGenerator {
    expression: String,
    input_source: InputSource,
    output_format: OutputFormat,
}

impl CodeGenerator {
    /// Create a new code generator for the given expression
    pub fn new(expression: String, input_source: InputSource, output_format: OutputFormat) -> Self {
        Self {
            expression,
            input_source,
            output_format,
        }
    }

    /// Generate complete Rust program from expression
    pub fn generate(&self) -> Result<String> {
        let mut code = String::new();

        // Add prelude imports
        code.push_str("use lob_prelude::*;\n");
        code.push_str("use std::collections::HashMap;\n");

        // Add serde_json import if using JSON output (from lob_prelude re-export)
        if matches!(
            self.output_format,
            OutputFormat::Json | OutputFormat::JsonLines
        ) {
            code.push_str("use lob_prelude::serde_json;\n");
        }

        // Add tabled import if using Table output
        if matches!(self.output_format, OutputFormat::Table) {
            code.push_str("use lob_prelude::tabled::builder::Builder;\n");
            code.push_str("use lob_prelude::tabled::settings::Style;\n");
        }

        code.push('\n');
        code.push_str("fn main() {\n");

        // Check if expression uses stdin (starts with '_')
        let uses_stdin = self.expression.trim().starts_with('_');

        // Generate input based on format and source
        let expression = if uses_stdin {
            self.generate_input(&mut code);
            self.expression.replacen('_', "stdin_data", 1)
        } else {
            self.expression.clone()
        };

        // User expression
        code.push_str(&format!("    let result = {};\n", expression));

        // Generate output based on format
        self.generate_output(&mut code);

        code.push_str("}\n");

        Ok(code)
    }

    /// Generate input code based on input source and format
    fn generate_input(&self, code: &mut String) {
        match self.input_source.format {
            InputFormat::Lines => {
                if self.input_source.is_stdin() {
                    code.push_str("    let stdin_data = input();\n");
                } else {
                    code.push_str("    let files: Vec<_> = std::env::args().skip(1).map(|p| std::path::PathBuf::from(p)).collect();\n");
                    code.push_str("    let stdin_data = input_from_files(&files);\n");
                }
            }
            InputFormat::Csv => {
                if self.input_source.is_stdin() {
                    code.push_str("    let stdin_data = input_csv();\n");
                } else {
                    code.push_str("    let files: Vec<_> = std::env::args().skip(1).map(|p| std::path::PathBuf::from(p)).collect();\n");
                    code.push_str("    let stdin_data = input_csv_from_files(&files);\n");
                }
            }
            InputFormat::Tsv => {
                if self.input_source.is_stdin() {
                    code.push_str("    let stdin_data = input_tsv();\n");
                } else {
                    code.push_str("    let files: Vec<_> = std::env::args().skip(1).map(|p| std::path::PathBuf::from(p)).collect();\n");
                    code.push_str("    let stdin_data = input_tsv_from_files(&files);\n");
                }
            }
            InputFormat::JsonLines => {
                if self.input_source.is_stdin() {
                    code.push_str("    let stdin_data = input_json();\n");
                } else {
                    code.push_str("    let files: Vec<_> = std::env::args().skip(1).map(|p| std::path::PathBuf::from(p)).collect();\n");
                    code.push_str("    let stdin_data = input_json_from_files(&files);\n");
                }
            }
        }
    }

    /// Generate output code based on output format
    fn generate_output(&self, code: &mut String) {
        let is_iter = !self.has_terminal_operation();

        match self.output_format {
            OutputFormat::Debug => {
                if is_iter {
                    code.push_str("    for item in result {\n");
                    code.push_str("        println!(\"{:?}\", item);\n");
                    code.push_str("    }\n");
                } else {
                    code.push_str("    println!(\"{:?}\", result);\n");
                }
            }
            OutputFormat::Json => {
                if is_iter {
                    code.push_str("    let items: Vec<_> = result.collect();\n");
                    code.push_str(
                        "    println!(\"{}\", serde_json::to_string_pretty(&items).unwrap());\n",
                    );
                } else {
                    code.push_str(
                        "    println!(\"{}\", serde_json::to_string(&result).unwrap());\n",
                    );
                }
            }
            OutputFormat::JsonLines => {
                if is_iter {
                    code.push_str("    for item in result {\n");
                    code.push_str(
                        "        println!(\"{}\", serde_json::to_string(&item).unwrap());\n",
                    );
                    code.push_str("    }\n");
                } else {
                    code.push_str(
                        "    println!(\"{}\", serde_json::to_string(&result).unwrap());\n",
                    );
                }
            }
            OutputFormat::Csv => {
                if is_iter {
                    code.push_str("    let items: Vec<_> = result.collect();\n");
                    code.push_str("    output_csv(&items);\n");
                } else {
                    code.push_str("    output_csv(&[result]);\n");
                }
            }
            OutputFormat::Table => {
                if is_iter {
                    code.push_str("    let items: Vec<_> = result.collect();\n");
                    code.push_str("    if !items.is_empty() {\n");
                    code.push_str("        let mut builder = Builder::default();\n");
                    code.push_str("        // Extract headers from first item\n");
                    code.push_str("        let mut headers: Vec<_> = items[0].keys().collect();\n");
                    code.push_str("        headers.sort();\n");
                    code.push_str(
                        "        builder.push_record(headers.iter().map(|k| k.as_str()));\n",
                    );
                    code.push_str("        // Add data rows\n");
                    code.push_str("        for item in &items {\n");
                    code.push_str("            let row: Vec<_> = headers.iter().map(|k| item.get(*k).map(|v| v.as_str()).unwrap_or(\"\")).collect();\n");
                    code.push_str("            builder.push_record(row);\n");
                    code.push_str("        }\n");
                    code.push_str(
                        "        let table = builder.build().with(Style::rounded()).to_string();\n",
                    );
                    code.push_str("        println!(\"{}\", table);\n");
                    code.push_str("    }\n");
                } else {
                    code.push_str("    let mut builder = Builder::default();\n");
                    code.push_str("    let mut headers: Vec<_> = result.keys().collect();\n");
                    code.push_str("    headers.sort();\n");
                    code.push_str("    builder.push_record(headers.iter().map(|k| k.as_str()));\n");
                    code.push_str("    let row: Vec<_> = headers.iter().map(|k| result.get(*k).map(|v| v.as_str()).unwrap_or(\"\")).collect();\n");
                    code.push_str("    builder.push_record(row);\n");
                    code.push_str(
                        "    let table = builder.build().with(Style::rounded()).to_string();\n",
                    );
                    code.push_str("    println!(\"{}\", table);\n");
                }
            }
        }
    }

    /// Check if expression has a terminal operation
    fn has_terminal_operation(&self) -> bool {
        let terminals = [
            ".collect(",
            ".count()",
            ".sum(",
            ".sum::",
            ".min()",
            ".max()",
            ".reduce(",
            ".fold(",
            ".fold_left(",
            ".first()",
            ".last()",
            ".to_list()",
            ".any(",
            ".all(",
        ];

        terminals.iter().any(|t| self.expression.contains(t))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_input() -> InputSource {
        InputSource::new(vec![], InputFormat::Lines)
    }

    #[test]
    fn generate_basic() {
        let gen = CodeGenerator::new(
            "_.filter(|x| x.contains(\"test\"))".to_string(),
            default_input(),
            OutputFormat::Debug,
        );
        let code = gen.generate().unwrap();

        assert!(code.contains("use lob_prelude::*;"));
        assert!(code.contains("let stdin_data = input();"));
        assert!(code.contains("let result = stdin_data.filter(|x| x.contains(\"test\"));"));
        assert!(code.contains("for item in result"));
    }

    #[test]
    fn generate_with_terminal() {
        let gen = CodeGenerator::new(
            "_.filter(|x| x.len() > 5).count()".to_string(),
            default_input(),
            OutputFormat::Debug,
        );
        let code = gen.generate().unwrap();

        assert!(code.contains("println!(\"{:?}\", result);"));
        assert!(!code.contains("for item in result"));
    }

    #[test]
    fn generate_without_stdin() {
        let gen = CodeGenerator::new(
            "lob(vec![1, 2, 3]).map(|x| x * 2)".to_string(),
            default_input(),
            OutputFormat::Debug,
        );
        let code = gen.generate().unwrap();

        assert!(!code.contains("let stdin_data = input();"));
        assert!(code.contains("let result = lob(vec![1, 2, 3]).map(|x| x * 2);"));
    }

    #[test]
    fn generate_csv_input() {
        let gen = CodeGenerator::new(
            "_.filter(|r| r[\"age\"].parse::<i32>().unwrap() > 18)".to_string(),
            InputSource::new(vec![], InputFormat::Csv),
            OutputFormat::Debug,
        );
        let code = gen.generate().unwrap();

        assert!(code.contains("let stdin_data = input_csv();"));
    }

    #[test]
    fn generate_json_output() {
        let gen = CodeGenerator::new("_.take(5)".to_string(), default_input(), OutputFormat::Json);
        let code = gen.generate().unwrap();

        assert!(code.contains("serde_json::to_string_pretty"));
    }

    #[test]
    fn generate_tsv_input() {
        let gen = CodeGenerator::new(
            "_.take(5)".to_string(),
            InputSource::new(vec![], InputFormat::Tsv),
            OutputFormat::Debug,
        );
        let code = gen.generate().unwrap();

        assert!(code.contains("let stdin_data = input_tsv();"));
    }

    #[test]
    fn generate_json_input() {
        let gen = CodeGenerator::new(
            "_.take(5)".to_string(),
            InputSource::new(vec![], InputFormat::JsonLines),
            OutputFormat::Debug,
        );
        let code = gen.generate().unwrap();

        assert!(code.contains("let stdin_data = input_json();"));
    }

    #[test]
    fn generate_csv_from_files() {
        use std::path::PathBuf;
        let gen = CodeGenerator::new(
            "_.count()".to_string(),
            InputSource::new(vec![PathBuf::from("test.csv")], InputFormat::Csv),
            OutputFormat::Debug,
        );
        let code = gen.generate().unwrap();

        assert!(code.contains("input_csv_from_files"));
    }

    #[test]
    fn generate_tsv_from_files() {
        use std::path::PathBuf;
        let gen = CodeGenerator::new(
            "_.count()".to_string(),
            InputSource::new(vec![PathBuf::from("test.tsv")], InputFormat::Tsv),
            OutputFormat::Debug,
        );
        let code = gen.generate().unwrap();

        assert!(code.contains("input_tsv_from_files"));
    }

    #[test]
    fn generate_json_from_files() {
        use std::path::PathBuf;
        let gen = CodeGenerator::new(
            "_.count()".to_string(),
            InputSource::new(vec![PathBuf::from("test.json")], InputFormat::JsonLines),
            OutputFormat::Debug,
        );
        let code = gen.generate().unwrap();

        assert!(code.contains("input_json_from_files"));
    }

    #[test]
    fn generate_jsonl_output() {
        let gen = CodeGenerator::new(
            "_.take(5)".to_string(),
            default_input(),
            OutputFormat::JsonLines,
        );
        let code = gen.generate().unwrap();

        assert!(code.contains("serde_json::to_string"));
    }

    #[test]
    fn generate_csv_output() {
        let gen = CodeGenerator::new("_.take(5)".to_string(), default_input(), OutputFormat::Csv);
        let code = gen.generate().unwrap();

        assert!(code.contains("output_csv"));
    }

    #[test]
    fn has_terminal_operation() {
        let gen = CodeGenerator::new(
            "_.count()".to_string(),
            default_input(),
            OutputFormat::Debug,
        );
        assert!(gen.has_terminal_operation());

        let gen = CodeGenerator::new(
            "_.collect()".to_string(),
            default_input(),
            OutputFormat::Debug,
        );
        assert!(gen.has_terminal_operation());

        let gen = CodeGenerator::new(
            "_.filter(|x| true)".to_string(),
            default_input(),
            OutputFormat::Debug,
        );
        assert!(!gen.has_terminal_operation());
    }

    #[test]
    fn generate_json_output_with_terminal() {
        let gen = CodeGenerator::new("_.count()".to_string(), default_input(), OutputFormat::Json);
        let code = gen.generate().unwrap();

        // Should use to_string not to_string_pretty for terminal operations
        assert!(code.contains("serde_json::to_string(&result)"));
        assert!(!code.contains("serde_json::to_string_pretty"));
    }

    #[test]
    fn generate_csv_output_with_terminal() {
        let gen = CodeGenerator::new("_.first()".to_string(), default_input(), OutputFormat::Csv);
        let code = gen.generate().unwrap();

        // Should wrap single result in array for CSV output
        assert!(code.contains("output_csv(&[result])"));
        assert!(!code.contains("result.collect()"));
    }
}
