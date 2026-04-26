mod tempdir;

use std::{
    fs::File,
    io::{self, Write},
    path::{Path, PathBuf},
};
use tempdir::TempDir;
use unrar::Archive;
use walkdir::WalkDir;
use zip::{ZipWriter, write::SimpleFileOptions};

fn zip_dir_to_cbz(src_dir: &Path, output_cbz: &Path) -> io::Result<()> {
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

        zip.start_file(name, options)?;
        let mut input = File::open(&path)?;
        io::copy(&mut input, &mut zip)?;
    }

    zip.finish()?;
    Ok(())
}

fn unrar<P>(tmp_dir: P) -> Result<(), io::Error>
where
    P: AsRef<Path>,
{
    let mut archive = match Archive::new("./Invincible.cbr").open_for_processing() {
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

        // println!(
        //     "{} bytes: {}",
        //     header.entry().unpacked_size,
        //     header.entry().filename.to_string_lossy(),
        // );

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

fn main() {
    let path = Path::new("./Invincible.cbr");

    let filename = path.file_stem();
    let extension = path.extension();
    return;

    let tmp_dir = match TempDir::new("cbr2cbz") {
        Ok(t) => t,
        Err(e) => {
            eprintln!("Could not create tmp dir: {e}");
            return;
        }
    };

    match unrar(&tmp_dir) {
        Ok(_) => println!("Unrar-ed"),
        Err(e) => eprintln!("Failed to unrar: {e}"),
    }

    let cbz_path = Path::new("Invincible.cbz");

    match zip_dir_to_cbz(tmp_dir.path(), cbz_path) {
        Ok(_) => println!("Created CBZ"),
        Err(e) => eprintln!("Failed to create CBZ: {e}"),
    }

    match tmp_dir.close() {
        Ok(_) => {
            println!("Deleted tmp dir");
        }
        Err(e) => {
            eprintln!("Failed to delete tmp dir: {e}");
        }
    }
}
