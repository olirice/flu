//! Lob - Embedded Rust Pipeline Tool
//!
//! A self-contained CLI for running Rust data pipeline one-liners.

#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod cache;
mod codegen;
mod compile;
mod error;
mod input;
mod output;
mod suggestion;
mod toolchain;
mod welcome;

use cache::Cache;
use clap::Parser;
use codegen::CodeGenerator;
use compile::Compiler;
use error::{LobError, Result};
use input::{InputFormat, InputSource};
use output::OutputFormat;
use std::path::PathBuf;
use std::process::Command;
use toolchain::EmbeddedToolchain;

/// Lob - Embedded Rust Pipeline Tool
#[derive(Parser, Debug)]
#[command(name = "lob")]
#[command(about = "Run Rust data pipeline one-liners", long_about = None)]
#[command(version)]
struct Args {
    /// Lob expression to execute
    #[arg(value_name = "EXPRESSION", required_unless_present_any = ["show_source", "clear_cache", "cache_stats"])]
    expression: Option<String>,

    /// Input files (omit to read from stdin)
    #[arg(value_name = "FILE")]
    files: Vec<PathBuf>,

    /// Parse input as CSV with headers (row is `HashMap<String, String>`)
    #[arg(long)]
    parse_csv: bool,

    /// Parse input as TSV with headers
    #[arg(long)]
    parse_tsv: bool,

    /// Parse input as JSON lines
    #[arg(long)]
    parse_json: bool,

    /// Output format
    #[arg(short = 'f', long, value_name = "FORMAT")]
    #[arg(value_parser = ["debug", "json", "jsonl", "csv", "table"])]
    format: Option<String>,

    /// Show generated source code without executing
    #[arg(short = 's', long)]
    show_source: bool,

    /// Clear the compilation cache
    #[arg(long)]
    clear_cache: bool,

    /// Show cache statistics
    #[arg(long)]
    cache_stats: bool,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Show performance statistics after execution
    #[arg(long)]
    stats: bool,
}

fn main() {
    if let Err(e) = run() {
        // Compilation errors are already formatted nicely
        match &e {
            LobError::Compilation(msg) => eprintln!("{}", msg),
            _ => eprintln!("Error: {}", e),
        }
        std::process::exit(1);
    }
}

/// Initialize the compiler, trying embedded toolchain first, then system rustc
fn initialize_compiler(verbose: bool) -> Result<Compiler> {
    // Try embedded toolchain first
    match EmbeddedToolchain::ensure_extracted() {
        Ok(toolchain) => {
            if toolchain.is_valid() {
                if verbose {
                    eprintln!("Using embedded Rust toolchain");
                }
                return Ok(Compiler::custom(
                    toolchain.rustc_path(),
                    Some(toolchain.sysroot()),
                ));
            } else if verbose {
                eprintln!("Embedded toolchain invalid, falling back to system rustc");
            }
        }
        Err(e) => {
            if verbose {
                eprintln!("Embedded toolchain not available: {}", e);
                eprintln!("Falling back to system rustc");
            }
        }
    }

    // Fall back to system rustc
    Compiler::system()
}

fn run() -> Result<()> {
    let args = Args::parse();

    // Handle cache management commands
    if args.clear_cache {
        let cache = Cache::new()?;
        cache.clear()?;
        println!("Cache cleared successfully");
        return Ok(());
    }

    if args.cache_stats {
        let cache = Cache::new()?;
        let stats = cache.stats()?;
        println!("Cache statistics:");
        println!("  Cached binaries: {}", stats.binary_count);
        println!("  Total size: {}", stats.format_size());
        println!("  Cache directory: {:?}", cache.cache_dir());
        return Ok(());
    }

    // Show welcome message if no expression and stdin is a terminal
    if args.expression.is_none() {
        if args.files.is_empty() && atty::is(atty::Stream::Stdin) {
            welcome::print_welcome();
            return Ok(());
        }
        return Err(LobError::InvalidExpression(
            "No expression provided. Use --help for usage.".to_string(),
        ));
    }

    let expression = args.expression.unwrap();

    // Determine input format
    let input_format = if args.parse_csv {
        InputFormat::Csv
    } else if args.parse_tsv {
        InputFormat::Tsv
    } else if args.parse_json {
        InputFormat::JsonLines
    } else {
        InputFormat::Lines
    };

    // Create input source
    let input_source = InputSource::new(args.files.clone(), input_format);
    input_source.validate()?;

    // Determine output format
    let output_format = if let Some(ref fmt) = args.format {
        OutputFormat::from_str(fmt)
            .ok_or_else(|| LobError::InvalidExpression(format!("Unknown output format: {}", fmt)))?
    } else {
        OutputFormat::default(output::is_terminal())
    };

    // Generate code
    let expression_clone = expression.clone();
    let generator = CodeGenerator::new(expression, input_source.clone(), output_format);
    let source = generator.generate()?;

    if args.show_source {
        println!("{}", source);
        return Ok(());
    }

    // Initialize cache and compiler
    let cache = Cache::new()?;
    let compiler = initialize_compiler(args.verbose)?;

    // Compile (with caching)
    if args.verbose {
        eprintln!("Compiling expression...");
    }

    let compile_start = std::time::Instant::now();
    let compile_result = compiler.compile_and_cache(&source, &cache, Some(&expression_clone))?;
    let compile_time = compile_start.elapsed();

    if args.verbose {
        eprintln!("Compiled binary: {:?}", compile_result.binary_path);
        eprintln!("Cache hit: {}", compile_result.cache_hit);
        eprintln!("Executing...");
    }

    // Execute the compiled binary
    let exec_start = std::time::Instant::now();
    let mut cmd = Command::new(&compile_result.binary_path);

    // Pass files as arguments if any
    if !input_source.is_stdin() {
        cmd.args(&input_source.files);
    }

    let mut child = cmd
        .stdin(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .spawn()?;

    let status = child.wait()?;
    let exec_time = exec_start.elapsed();
    let total_time = compile_start.elapsed();

    if !status.success() {
        return Err(LobError::Compilation(format!(
            "Execution failed with status: {}",
            status
        )));
    }

    // Display statistics if requested
    if args.stats {
        eprintln!();
        eprintln!("Statistics:");
        eprintln!("  Compilation time: {:?}", compile_time);
        eprintln!("  Execution time:   {:?}", exec_time);
        eprintln!("  Total time:       {:?}", total_time);
        eprintln!(
            "  Cache:            {}",
            if compile_result.cache_hit {
                "Hit (binary reused)"
            } else {
                "Miss (compiled)"
            }
        );
    }

    Ok(())
}
