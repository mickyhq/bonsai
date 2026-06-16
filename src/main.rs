mod budget;
mod formatter;
mod parser;
mod walker;

use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};

use budget::{count_text_tokens, optimize_budget, ProcessedFile};
use formatter::format_repository_context;
use parser::{compress_file, CompressionLevel};
use walker::collect_code_files;

#[derive(Debug, Parser)]
#[command(name = "contextshrink")]
#[command(about = "Shrink repository source context into token-efficient XML")]
struct Cli {
    #[arg(default_value = ".")]
    path: PathBuf,

    #[arg(long, default_value_t = 4000)]
    max_tokens: usize,

    #[arg(long, default_value_t = 2)]
    level: u8,

    #[arg(long, value_enum, default_value_t = OutputDestination::File)]
    output: OutputDestination,

    #[arg(long, default_value = "contextshrink.xml")]
    output_file: PathBuf,

    #[arg(long)]
    stats: bool,

    #[arg(long)]
    summary: bool,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum OutputDestination {
    Clipboard,
    File,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let root = fs::canonicalize(&cli.path)
        .with_context(|| format!("cannot resolve target path {}", cli.path.display()))?;
    let requested_level = CompressionLevel::try_from(cli.level)?;

    let paths = collect_code_files(&root)?;
    let mut files = Vec::with_capacity(paths.len());

    for path in paths {
        let relative_path = path
            .strip_prefix(&root)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");

        let variants = compress_file(&path, requested_level)
            .with_context(|| format!("failed to parse {}", path.display()))?;

        files.push(ProcessedFile::new(relative_path, requested_level, variants));
    }

    let raw_xml = format_repository_context(&full_context_files(&files));
    let optimized = optimize_budget(files, cli.max_tokens)?;
    let xml = format_repository_context(&optimized);
    let run_stats = RunStats::new(&cli, requested_level, optimized.len(), &raw_xml, &xml)?;

    match cli.output {
        OutputDestination::Clipboard => {
            let mut clipboard = arboard::Clipboard::new().context("cannot access clipboard")?;
            clipboard.set_text(xml).context("cannot write clipboard")?;
        }
        OutputDestination::File => {
            fs::write(&cli.output_file, xml)
                .with_context(|| format!("cannot write {}", cli.output_file.display()))?;
        }
    }

    if cli.summary {
        print_summary(&run_stats);
    }

    if cli.stats {
        print_stats(&run_stats);
    }

    Ok(())
}

#[derive(Debug)]
struct RunStats {
    output_target: String,
    files_scanned: usize,
    selected_level: CompressionLevel,
    max_tokens: usize,
    raw_tokens: usize,
    shrunk_tokens: usize,
    tokens_saved: usize,
    saving_percent: f64,
}

impl RunStats {
    fn new(
        cli: &Cli,
        selected_level: CompressionLevel,
        files_scanned: usize,
        raw_xml: &str,
        shrunk_xml: &str,
    ) -> Result<Self> {
        let raw_tokens = count_text_tokens(raw_xml)?;
        let shrunk_tokens = count_text_tokens(shrunk_xml)?;
        let tokens_saved = raw_tokens.saturating_sub(shrunk_tokens);
        let saving_percent = if raw_tokens == 0 {
            0.0
        } else {
            tokens_saved as f64 / raw_tokens as f64 * 100.0
        };

        Ok(Self {
            output_target: output_target(cli),
            files_scanned,
            selected_level,
            max_tokens: cli.max_tokens,
            raw_tokens,
            shrunk_tokens,
            tokens_saved,
            saving_percent,
        })
    }
}

fn full_context_files(files: &[ProcessedFile]) -> Vec<ProcessedFile> {
    files
        .iter()
        .cloned()
        .map(|mut file| {
            file.level = CompressionLevel::Full;
            file
        })
        .collect()
}

fn output_target(cli: &Cli) -> String {
    match cli.output {
        OutputDestination::Clipboard => "clipboard".to_owned(),
        OutputDestination::File => cli.output_file.display().to_string(),
    }
}

fn print_summary(stats: &RunStats) {
    println!("summary:");
    println!("  output: {}", stats.output_target);
    println!("  files_included: {}", stats.files_scanned);
    println!("  selected_level: {}", stats.selected_level.as_u8());
    println!("  output_tokens: {} / {}", stats.shrunk_tokens, stats.max_tokens);
}

fn print_stats(stats: &RunStats) {
    println!("stats:");
    println!("  raw_tokens: {}", stats.raw_tokens);
    println!("  shrunk_tokens: {}", stats.shrunk_tokens);
    println!("  tokens_saved: {}", stats.tokens_saved);
    println!("  saving_percent: {:.2}", stats.saving_percent);
    println!("  files_scanned: {}", stats.files_scanned);
}
