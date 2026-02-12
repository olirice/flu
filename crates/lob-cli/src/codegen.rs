//! Code generation for lob expressions

use crate::error::Result;
use crate::input::{InputFormat, InputSource};
use crate::output::OutputFormat;

/// Generates Rust source code from a lob expression
pub struct CodeGenerator {
    expression: String,
    input_source: InputSource,
    output_format: OutputFormat,
    enable_stats: bool,
}

impl CodeGenerator {
    /// Create a new code generator for the given expression
    pub fn new(
        expression: String,
        input_source: InputSource,
        output_format: OutputFormat,
        enable_stats: bool,
    ) -> Self {
        Self {
            expression,
            input_source,
            output_format,
            enable_stats,
        }
    }

    /// Generate complete Rust program from expression
    pub fn generate(&self) -> Result<String> {
        let mut code = String::new();

        // Add prelude imports
        code.push_str("use lob_prelude::*;\n");
        code.push_str("use std::collections::HashMap;\n");

        // Add stats tracking imports if enabled
        if self.enable_stats {
            code.push_str("use std::sync::atomic::{AtomicUsize, Ordering};\n");
            code.push_str("use std::sync::Arc;\n");
            code.push_str("use std::time::Instant;\n");
        }

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

        // Initialize stats tracking if enabled
        if self.enable_stats {
            code.push_str("    let start_time = Instant::now();\n");
            code.push_str("    let item_count = Arc::new(AtomicUsize::new(0));\n");
            code.push_str("    let last_print = Arc::new(AtomicUsize::new(0));\n");
            code.push_str("    let print_interval = 10000; // Print every 10k items\n");
            code.push('\n');
        }

        // Check if expression uses stdin (starts with '_')
        let uses_stdin = self.expression.trim().starts_with('_');

        // Generate input based on format and source
        let expression = if uses_stdin {
            self.generate_input(&mut code);
            if self.enable_stats {
                // Wrap iterator with stats tracking
                code.push_str("    let stdin_data = {\n");
                code.push_str("        let counter = item_count.clone();\n");
                code.push_str("        let last = last_print.clone();\n");
                code.push_str("        let start = start_time;\n");
                code.push_str("        stdin_data.map(move |item| {\n");
                code.push_str(
                    "            let count = counter.fetch_add(1, Ordering::Relaxed) + 1;\n",
                );
                code.push_str("            let last_val = last.load(Ordering::Relaxed);\n");
                code.push_str("            if count - last_val >= print_interval {\n");
                code.push_str("                let elapsed = start.elapsed().as_secs_f64();\n");
                code.push_str("                let throughput = count as f64 / elapsed;\n");
                code.push_str(
                    "                eprintln!(\"\\r[Stats] Items: {} | Throughput: {:.0} items/s | Elapsed: {:.1}s\", count, throughput, elapsed);\n",
                );
                code.push_str("                last.store(count, Ordering::Relaxed);\n");
                code.push_str("            }\n");
                code.push_str("            item\n");
                code.push_str("        })\n");
                code.push_str("    };\n");
            }
            self.expression.replacen('_', "stdin_data", 1)
        } else {
            self.expression.clone()
        };

        // User expression
        code.push_str(&format!("    let result = {};\n", expression));

        // Generate output based on format
        self.generate_output(&mut code);

        // Print final stats if enabled
        if self.enable_stats {
            code.push('\n');
            code.push_str("    let total_items = item_count.load(Ordering::Relaxed);\n");
            code.push_str("    let elapsed = start_time.elapsed().as_secs_f64();\n");
            code.push_str("    let throughput = if elapsed > 0.0 { total_items as f64 / elapsed } else { 0.0 };\n");
            code.push_str(
                "    eprintln!(\"\\n[Final Stats] Total items: {} | Throughput: {:.0} items/s | Total time: {:.3}s\", total_items, throughput, elapsed);\n",
            );
        }

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
