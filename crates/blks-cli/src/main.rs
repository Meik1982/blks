//! blks - High-Performance 384-Bit Cryptographic Tree-Hash CLI
//!
//! Produces 384-bit digests that encode to exactly 64 Base64 characters without padding,
//! breaking the 128-bit collision limit of BLAKE3/SHA-256 with 192 bits of collision resistance.

use blks_core::{constant_time_eq, hash_file_opts, hash_reader, Digest};
use clap::{Parser, ValueEnum};
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;

#[derive(Clone, Copy, Debug, ValueEnum, PartialEq, Eq)]
enum OutputFormat {
    /// 64-character standard Base64 (matches SHA-256 hex line width, zero padding)
    Base64,
    /// 64-character URL-safe Base64 (using - and _ instead of + and /)
    Base64Url,
    /// 96-character hexadecimal digest
    Hex,
    /// Raw 48-byte binary digest to stdout
    Raw,
}

#[derive(Parser, Debug)]
#[command(
    name = "blks",
    about = "High-Performance 384-Bit Cryptographic Tree-Hash Utility (64-Char Base64, 192-bit Collision Resistance)",
    version
)]
struct Cli {
    /// Files to hash (reads from stdin if omitted or '-')
    #[arg(value_name = "FILE")]
    files: Vec<PathBuf>,

    /// Read checksums from the specified file and check them
    #[arg(short = 'c', long = "check", value_name = "FILE")]
    check: Option<PathBuf>,

    /// Digest output format
    #[arg(long = "format", value_enum, default_value_t = OutputFormat::Base64)]
    format: OutputFormat,

    /// Convenience flag for hexadecimal output (equivalent to --format hex)
    #[arg(long = "hex")]
    hex: bool,

    /// Convenience flag for raw binary output to stdout
    #[arg(long = "raw")]
    raw: bool,

    /// Number of worker threads for parallel tree reduction (default: all logical cores)
    #[arg(short = 'j', long = "threads")]
    threads: Option<usize>,

    /// Quiet mode: in check mode, only print failed files
    #[arg(short = 'q', long = "quiet")]
    quiet: bool,

    /// Output machine-readable JSON telemetry
    #[arg(long = "json")]
    json: bool,

    /// Benchmark mode: print elapsed time and throughput in GB/s
    #[arg(long = "benchmark")]
    benchmark: bool,

    /// Disable memory-mapping (forces streaming reads; prevents SIGBUS on network shares or volatile files)
    #[arg(long = "no-mmap")]
    no_mmap: bool,
}

fn format_digest(digest: &Digest, format: OutputFormat) -> String {
    match format {
        OutputFormat::Base64 => digest.to_base64(),
        OutputFormat::Base64Url => digest.to_base64_url(),
        OutputFormat::Hex => digest.to_hex(),
        OutputFormat::Raw => String::new(),
    }
}

fn hash_stdin() -> io::Result<Digest> {
    let stdin = io::stdin();
    let bytes = hash_reader(stdin.lock())?;
    Ok(Digest(bytes))
}

fn run_check(check_file: &Path, quiet: bool, use_mmap: bool) -> io::Result<bool> {
    let file = File::open(check_file)?;
    let reader = BufReader::new(file);

    let mut total = 0;
    let mut failures = 0;

    for (line_num, line_res) in reader.lines().enumerate() {
        let line = line_res?;
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        // Expected format: "<HASH>  <FILENAME>" or "<HASH> *<FILENAME>"
        let parts: Vec<&str> = trimmed.splitn(2, |c: char| c.is_whitespace()).collect();
        if parts.len() < 2 {
            eprintln!(
                "blks: {}:{}: invalid check line format",
                check_file.display(),
                line_num + 1
            );
            failures += 1;
            continue;
        }

        let expected_hash = parts[0].trim();
        let filepath = parts[1].trim().trim_start_matches('*').trim();

        total += 1;
        match hash_file_opts(filepath, use_mmap) {
            Ok(computed) => {
                let matches = if expected_hash.len() == 64 {
                    if let Ok(expected_digest) = Digest::from_base64(expected_hash) {
                        computed.ct_eq(&expected_digest)
                    } else {
                        false
                    }
                } else if expected_hash.len() == 96 {
                    constant_time_eq(
                        computed.to_hex().as_bytes(),
                        expected_hash.to_ascii_lowercase().as_bytes(),
                    )
                } else {
                    false
                };

                if matches {
                    if !quiet {
                        println!("{}: OK", filepath);
                    }
                } else {
                    println!("{}: FAILED", filepath);
                    failures += 1;
                }
            }
            Err(e) => {
                println!("{}: FAILED (cannot read: {})", filepath, e);
                failures += 1;
            }
        }
    }

    if failures > 0 {
        eprintln!(
            "blks: WARNING: {} of {} computed checksums did NOT match",
            failures, total
        );
        Ok(false)
    } else {
        Ok(true)
    }
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    if let Some(num_threads) = cli.threads {
        let _ = rayon::ThreadPoolBuilder::new()
            .num_threads(num_threads)
            .build_global();
    }

    let format = if cli.raw {
        OutputFormat::Raw
    } else if cli.hex {
        OutputFormat::Hex
    } else {
        cli.format
    };

    if let Some(check_path) = &cli.check {
        match run_check(check_path, cli.quiet, !cli.no_mmap) {
            Ok(true) => return ExitCode::SUCCESS,
            _ => return ExitCode::FAILURE,
        }
    }

    let files = if cli.files.is_empty() {
        vec![PathBuf::from("-")]
    } else {
        cli.files
    };

    let mut had_error = false;

    for path in &files {
        let is_stdin = path.as_os_str() == "-";
        let display_name = if is_stdin {
            "-"
        } else {
            path.to_str().unwrap_or("?")
        };

        let start_time = Instant::now();
        let result = if is_stdin {
            hash_stdin()
        } else {
            hash_file_opts(path, !cli.no_mmap)
        };

        match result {
            Ok(digest) => {
                let elapsed = start_time.elapsed();
                if format == OutputFormat::Raw {
                    let mut stdout = io::stdout().lock();
                    let _ = io::Write::write_all(&mut stdout, digest.as_bytes());
                } else if cli.json {
                    let file_size = if is_stdin {
                        None
                    } else {
                        path.metadata().map(|m| m.len()).ok()
                    };
                    println!(
                        r#"{{"file":"{}","hash":"{}","algo":"blks-384","bits":384,"collision_bits":192,"elapsed_ms":{:.3}{}}}"#,
                        display_name,
                        format_digest(&digest, format),
                        elapsed.as_secs_f64() * 1000.0,
                        file_size
                            .map(|s| format!(r#","bytes":{}"#, s))
                            .unwrap_or_default()
                    );
                } else {
                    let formatted = format_digest(&digest, format);
                    if cli.benchmark {
                        let file_size = path.metadata().map(|m| m.len()).unwrap_or(0);
                        let secs = elapsed.as_secs_f64();
                        let throughput_gbs = if secs > 0.0 {
                            (file_size as f64) / (1024.0 * 1024.0 * 1024.0) / secs
                        } else {
                            0.0
                        };
                        println!(
                            "{}  {}  [{:.2} GB/s, {:.3} ms]",
                            formatted,
                            display_name,
                            throughput_gbs,
                            secs * 1000.0
                        );
                    } else {
                        println!("{}  {}", formatted, display_name);
                    }
                }
            }
            Err(e) => {
                eprintln!("blks: {}: {}", display_name, e);
                had_error = true;
            }
        }
    }

    if had_error {
        ExitCode::FAILURE
    } else {
        ExitCode::SUCCESS
    }
}
