//! Welcome message and help display

use colored::Colorize;

/// Print welcome message when lob is run without arguments
pub fn print_welcome() {
    println!("{}\n", "lob - Embedded Rust Pipeline Tool".bold().cyan());

    println!("{}", "USAGE:".bold());
    println!("    lob [OPTIONS] [FILE...] <EXPRESSION>");
    println!("    command | lob [OPTIONS] <EXPRESSION>\n");

    println!("{}", "EXAMPLES:".bold());
    println!("    {}", "# Filter even numbers".dimmed());
    println!("    seq 1 100 | lob '_.filter(|x| x.parse::<i32>().unwrap() % 2 == 0).count()'");
    println!("    {}\n", "# Output: 50".dimmed());

    println!("    {}", "# Process file directly".dimmed());
    println!("    lob data.txt '_.filter(|x| x.len() > 5).take(10)'");
    println!(
        "    {}\n",
        "# Output: First 10 lines longer than 5 chars".dimmed()
    );

    println!("    {}", "# Parse CSV data".dimmed());
    println!(
        "    lob users.csv --parse-csv '_.filter(|r| r[\"age\"].parse::<i32>().unwrap() > 18)'"
    );
    println!("    {}\n", "# Output: Rows where age > 18".dimmed());

    println!("    {}", "# Multiple files".dimmed());
    println!("    lob file1.txt file2.txt '_.unique().count()'\n");

    println!("{}", "COMMON OPERATIONS:".bold());
    println!(
        "    {}  filter, take, skip, unique, drop_while",
        "Selection: ".cyan()
    );
    println!("    {}  map, enumerate, zip, flatten", "Transform: ".cyan());
    println!("    {}  chunk, window, group_by", "Grouping:  ".cyan());
    println!(
        "    {}  count, sum, min, max, to_list",
        "Terminal:  ".cyan()
    );
    println!();

    println!("{}", "INPUT FORMATS:".bold());
    println!("    --parse-csv         Parse input as CSV with headers");
    println!("    --parse-tsv         Parse input as TSV with headers");
    println!("    --parse-json        Parse each line as JSON");
    println!();

    println!("{}", "OUTPUT FORMATS:".bold());
    println!("    --format debug      Rust debug format (default)");
    println!("    --format json       JSON array");
    println!("    --format jsonl      JSON lines (one per line)");
    println!("    --format csv        CSV output (requires CSV input)");
    println!("    --format table      Table output (requires CSV/JSON input)");
    println!();

    println!("{}", "LEARN MORE:".bold());
    println!("    lob --help              Full documentation");
    println!("    lob --show-source EXPR  See generated Rust code");
    println!("    lob --cache-stats       View compilation cache");
    println!();

    println!("{}", "https://github.com/olirice/lob".dimmed());
}
