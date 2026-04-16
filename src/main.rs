mod enhancer;
mod prompt;
mod security;
mod vertex;

use anyhow::{Context, Result};
use clap::Parser;
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser)]
#[command(
    name = "wisdomguard",
    version,
    about = "Enhance agentic AI guides with VertexAI Gemini insights"
)]
struct Cli {
    /// Path to the IR JSON file (from cliguard, apiguard, or docguard --output-format json)
    ir_file: PathBuf,

    /// Base guide markdown to merge enhancements into (from cliguard, apiguard, or docguard)
    #[arg(long)]
    base_guide: Option<PathBuf>,

    /// Gemini model to use
    #[arg(long, default_value = "gemini-2.5-flash")]
    model: String,

    /// Google Cloud project ID (or set `GOOGLE_CLOUD_PROJECT`)
    #[arg(long)]
    project: Option<String>,

    /// Vertex AI location (or set `VERTEX_AI_LOCATION`)
    #[arg(long)]
    location: Option<String>,

    /// Output file path (stdout if not specified)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Output format: md or json
    #[arg(long, default_value = "md")]
    output_format: OutputFormat,

    /// Show prompts without calling the API
    #[arg(long)]
    dry_run: bool,
}

#[derive(Clone, clap::ValueEnum)]
enum OutputFormat {
    Md,
    Json,
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            let msg = format!("{e:#}");
            if msg.contains("Authentication") || msg.contains("credentials") {
                eprintln!("Error: {msg}");
                ExitCode::from(3)
            } else if msg.contains("Cannot open")
                || msg.contains("symbolic link")
                || msg.contains("traversal")
            {
                eprintln!("Error: {msg}");
                ExitCode::from(2)
            } else {
                eprintln!("Error: {msg}");
                ExitCode::from(1)
            }
        }
    }
}

#[tokio::main]
async fn run() -> Result<()> {
    let cli = Cli::parse();

    let ir_json = security::load_file_safe(&cli.ir_file).context("Could not load IR file")?;

    let _: serde_json::Value =
        serde_json::from_str(&ir_json).context("IR file is not valid JSON")?;

    if cli.dry_run {
        let prompt_text = enhancer::dry_run_prompt(&ir_json);
        println!("{prompt_text}");
        return Ok(());
    }

    let config = vertex::VertexConfig::from_args(
        cli.project.as_deref(),
        cli.location.as_deref(),
        Some(&cli.model),
    )?;

    let client = vertex::VertexClient::new(config).await?;

    // Get LLM enhancements first, then format based on output mode
    let enhancements = enhancer::get_enhancements(&ir_json, &client).await?;

    let output = match cli.output_format {
        OutputFormat::Json => serde_json::to_string_pretty(&enhancements)
            .context("Failed to serialize enhancements")?,
        OutputFormat::Md => {
            if let Some(ref base_path) = cli.base_guide {
                let base =
                    security::load_file_safe(base_path).context("Could not load base guide")?;
                enhancer::merge_enhancements_into(&base, &enhancements)
            } else {
                enhancer::format_standalone(&ir_json, &enhancements)
            }
        }
    };

    if let Some(ref path) = cli.output {
        security::write_output_safe(path, &output).context("Failed to write output file")?;
        eprintln!("Enhanced guide written to output file");
    } else {
        print!("{output}");
    }

    Ok(())
}
