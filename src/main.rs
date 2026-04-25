mod tempdir;

use tempdir::TempDir;
use std::fs::File;
use std::io::Write;

fn main() {
    if let Err(_) = run() {
        ::std::process::exit(1);
    }
}

use std::io;

fn run() -> Result<(), io::Error> {
    // Create a directory inside of `std::env::temp_dir()`, named with
    // the prefix "example".
    let tmp_dir = TempDir::new("example")?;
    let file_path = tmp_dir.path().join("my-temporary-note.txt");
    let mut tmp_file = File::create(file_path)?;
    writeln!(tmp_file, "Brian was here. Briefly.")?;

    println!("Press Enter to continue...");
    let mut guess = String::new();
    io::stdin().read_line(&mut guess).expect("Failed to read line");

    // By closing the `TempDir` explicitly, we can check that it has
    // been deleted successfully. If we don't close it explicitly,
    // the directory will still be deleted when `tmp_dir` goes out
    // of scope, but we won't know whether deleting the directory
    // succeeded.
    drop(tmp_file);
    tmp_dir.close()?;
    Ok(())
}