use std::fs;
use std::path::Path;

use mdbook_driver::MDBook;

pub fn build(book_dir: &Path, output_dir: &Path) -> anyhow::Result<()> {
    println!("Building mdBook to {}", output_dir.display());

    let mut book = MDBook::load(book_dir)?;
    book.config.build.build_dir = output_dir.to_path_buf();
    book.build()?;

    let gitignore_path = output_dir.join(".gitignore");
    fs::write(&gitignore_path, "*")?;

    println!("mdBook built successfully");
    Ok(())
}

pub fn build_workspace_book(workspace_root: &Path) -> anyhow::Result<()> {
    build(
        &workspace_root.join("book"),
        &workspace_root.join("web").join("public").join("book"),
    )
}
