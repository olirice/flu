//! Code generation for flu expressions

use crate::error::Result;

/// Generates Rust source code from a flu expression
pub struct CodeGenerator {
    expression: String,
}

impl CodeGenerator {
    /// Create a new code generator for the given expression
    pub fn new(expression: String) -> Self {
        Self { expression }
    }

    /// Generate complete Rust program from expression
    pub fn generate(&self) -> Result<String> {
        let mut code = String::new();

        // Add prelude imports
        code.push_str("use flu_prelude::*;\n\n");

        code.push_str("fn main() {\n");

        // Check if expression uses stdin (starts with '_')
        let uses_stdin = self.expression.trim().starts_with('_');

        // Replace _ with actual stdin variable in expression
        let expression = if uses_stdin {
            code.push_str("    let stdin_data = input();\n");
            self.expression.replacen('_', "stdin_data", 1)
        } else {
            self.expression.clone()
        };

        // User expression
        code.push_str(&format!("    let result = {};\n", expression));

        // Auto-append output loop if not a terminal operation
        if !self.has_terminal_operation() {
            code.push_str("    for item in result {\n");
            code.push_str("        println!(\"{:?}\", item);\n");
            code.push_str("    }\n");
        } else {
            code.push_str("    println!(\"{:?}\", result);\n");
        }

        code.push_str("}\n");

        Ok(code)
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

    /// Get the expression
    #[allow(dead_code)]
    pub fn expression(&self) -> &str {
        &self.expression
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_basic() {
        let gen = CodeGenerator::new("_.filter(|x| x.contains(\"test\"))".to_string());
        let code = gen.generate().unwrap();

        assert!(code.contains("use flu_prelude::*;"));
        assert!(code.contains("let stdin_data = input();"));
        assert!(code.contains("let result = stdin_data.filter(|x| x.contains(\"test\"));"));
        assert!(code.contains("for item in result"));
    }

    #[test]
    fn generate_with_terminal() {
        let gen = CodeGenerator::new("_.filter(|x| x.len() > 5).count()".to_string());
        let code = gen.generate().unwrap();

        assert!(code.contains("println!(\"{:?}\", result);"));
        assert!(!code.contains("for item in result"));
    }

    #[test]
    fn generate_without_stdin() {
        let gen = CodeGenerator::new("flu(vec![1, 2, 3]).map(|x| x * 2)".to_string());
        let code = gen.generate().unwrap();

        assert!(!code.contains("let _ = input();"));
        assert!(code.contains("let result = flu(vec![1, 2, 3]).map(|x| x * 2);"));
    }

    #[test]
    fn has_terminal_operation() {
        let gen = CodeGenerator::new("_.count()".to_string());
        assert!(gen.has_terminal_operation());

        let gen = CodeGenerator::new("_.collect()".to_string());
        assert!(gen.has_terminal_operation());

        let gen = CodeGenerator::new("_.filter(|x| true)".to_string());
        assert!(!gen.has_terminal_operation());
    }
}
