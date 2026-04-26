mod tempdir;

use clap::Parser;
use glob::glob;
use std::{
    fs::{self, File},
    io,
    path::{Path, PathBuf},
};
use tempdir::TempDir;
use unrar::Archive;
use walkdir::WalkDir;
use zip::{ZipWriter, write::SimpleFileOptions};

#[derive(Parser, Debug)]
#[command(name = "cbr2cbz", about = "Convert CBR files to CBZ", version)]
struct Args {
    /// Input CBR file(s); supports wildcards (e.g. "*.cbr")
    #[arg(required = true)]
    inputs: Vec<String>,

    /// Custom output filename (only valid with a single input)
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Delete input file(s) after a successful conversion
    #[arg(short, long, default_value_t = true)]
    delete: bool,

    /// Enable verbose / debug output
    #[arg(short, long)]
    verbose: bool,
}

fn zip_dir_to_cbz(src_dir: &Path, output_cbz: &Path, verbose: bool) -> io::Result<()> {
    let file = File::create(output_cbz)?;
    let mut zip = ZipWriter::new(file);

    let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);

    let mut files: Vec<PathBuf> = WalkDir::new(src_dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| e.file_type().is_file())
        .map(|e| e.path().to_path_buf())
        .collect();

    files.sort();

    for path in files {
        let name = path
            .strip_prefix(src_dir)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");

        if verbose {
            println!("  Adding to CBZ: {name}");
        }

        zip.start_file(name, options)?;
        let mut input = File::open(&path)?;
        io::copy(&mut input, &mut zip)?;
    }

    zip.finish()?;
    Ok(())
}

fn unrar<P, Q>(input_path: P, tmp_dir: Q, verbose: bool) -> Result<(), io::Error>
where
    P: AsRef<Path>,
    Q: AsRef<Path>,
{
    let mut archive = match Archive::new(input_path.as_ref()).open_for_processing() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("Could not open archive: {e}");
            return Ok(());
        }
    };

    loop {
        let header = match archive.read_header() {
            Ok(Some(h)) => h,
            Ok(None) => break,
            Err(e) => {
                eprintln!("Could not read archive header: {e}");
                break;
            }
        };

        if verbose {
            println!(
                "  {} bytes: {}",
                header.entry().unpacked_size,
                header.entry().filename.to_string_lossy(),
            );
        }

        archive = if header.entry().is_file() {
            match header.extract_with_base(&tmp_dir) {
                Ok(a) => a,
                Err(e) => {
                    eprintln!("Could not extract file: {e}");
                    break;
                }
            }
        } else {
            match header.skip() {
                Ok(a) => a,
                Err(e) => {
                    eprintln!("Could not skip entry: {e}");
                    return Ok(());
                }
            }
        };
    }
    Ok(())
}

fn process_file(
    input_path: &Path,
    output_override: Option<&Path>,
    delete: bool,
    verbose: bool,
) -> io::Result<()> {
    // Check that the file exists
    if !input_path.exists() {
        eprintln!("File not found: {}", input_path.display());
        return Ok(());
    }

    if !input_path.is_file() {
        eprintln!("Not a file: {}", input_path.display());
        return Ok(());
    }

    // Check that the input is a CBR file
    let extension = input_path
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_lowercase());
    if extension.as_deref() != Some("cbr") {
        eprintln!("Skipping non-CBR file: {}", input_path.display());
        return Ok(());
    }

    let stem = match input_path.file_stem().and_then(|s| s.to_str()) {
        Some(s) => s.to_string(),
        None => {
            eprintln!("Invalid filename: {}", input_path.display());
            return Ok(());
        }
    };

    println!("Processing: {}", input_path.display());

    // Create a tmp dir using the input filename as prefix
    let tmp_dir = match TempDir::new(&stem) {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Could not create tmp dir: {e}");
            return Ok(());
        }
    };

    if verbose {
        println!("Created tmp dir: {}", tmp_dir.path().display());
    }

    // Use a sub-directory for the extracted contents so the in-progress
    // CBZ (created in tmp_dir itself) doesn't get walked into the zip.
    let extract_dir = tmp_dir.path().join("extracted");
    if let Err(e) = fs::create_dir(&extract_dir) {
        eprintln!("Could not create extract dir: {e}");
        return Ok(());
    }

    if verbose {
        println!("Extracting CBR contents...");
    }

    if let Err(e) = unrar(input_path, &extract_dir, verbose) {
        eprintln!("Failed to unrar: {e}");
        return Ok(());
    }

    if verbose {
        println!("Building CBZ in tmp dir...");
    }

    // Build the CBZ inside the tmp dir first
    let tmp_cbz = tmp_dir.path().join(format!("{stem}.cbz"));
    if let Err(e) = zip_dir_to_cbz(&extract_dir, &tmp_cbz, verbose) {
        eprintln!("Failed to create CBZ: {e}");
        return Ok(());
    }

    // Determine the final output path
    let final_output = match output_override {
        Some(p) => p.to_path_buf(),
        None => input_path.with_extension("cbz"),
    };

    if verbose {
        println!("Copying CBZ to: {}", final_output.display());
    }

    // Copy the file over to the correct dir
    if let Err(e) = fs::copy(&tmp_cbz, &final_output) {
        eprintln!("Failed to copy CBZ to final location: {e}");
        return Ok(());
    }

    println!("Created CBZ: {}", final_output.display());

    // Clean up the tmp dir
    match tmp_dir.close() {
        Ok(_) => {
            if verbose {
                println!("Deleted tmp dir");
            }
        }
        Err(e) => {
            eprintln!("Failed to delete tmp dir: {e}");
        }
    }

    // Optionally delete the input file (only on success path)
    if delete {
        match fs::remove_file(input_path) {
            Ok(_) => println!("Deleted input file: {}", input_path.display()),
            Err(e) => eprintln!("Failed to delete input file: {e}"),
        }
    }

    Ok(())
}

/// Expand a single input string into a list of paths, handling wildcards.
fn expand_input(input: &str) -> Vec<PathBuf> {
    if input.contains('*') || input.contains('?') || input.contains('[') {
        match glob(input) {
            Ok(paths) => {
                let results: Vec<PathBuf> = paths.filter_map(Result::ok).collect();
                if results.is_empty() {
                    eprintln!("No files matched pattern: {input}");
                }
                results
            }
            Err(e) => {
                eprintln!("Invalid glob pattern '{input}': {e}");
                Vec::new()
            }
        }
    } else {
        vec![PathBuf::from(input)]
    }
}

fn main() {
    let args = Args::parse();

    // Expand wildcards across all inputs
    let mut files: Vec<PathBuf> = Vec::new();
    for input in &args.inputs {
        files.extend(expand_input(input));
    }

    if files.is_empty() {
        eprintln!("No input files to process");
        std::process::exit(1);
    }

    // --output only makes sense with a single input
    if args.output.is_some() && files.len() > 1 {
        eprintln!("--output can only be used with a single input file");
        std::process::exit(1);
    }

    if args.verbose {
        println!("Found {} file(s) to process", files.len());
    }

    for file in &files {
        if let Err(e) = process_file(file, args.output.as_deref(), args.delete, args.verbose) {
            eprintln!("Error processing {}: {e}", file.display());
        }
    }
}