use std::fs;
use std::path::Path;

use anyhow::Context;
use bon::Builder;
use mdbook_driver::MDBook;
use mdbook_driver::book::BookItem;
use path_slash::PathExt as _;

#[derive(Builder)]
pub struct LlmsConfig<'a> {
    pub book_root: &'a Path,
    pub llms_path: &'a Path,
    pub llms_full_path: &'a Path,
    pub llms_markdown_dir: &'a Path,
    pub base_url: &'a str,
    pub header: &'a str,
    pub markdown_dir_name: &'a str,
}

pub fn build_workspace_llms(
    workspace_root: &Path,
    base_url: &str,
    header: &str,
    markdown_dir_name: Option<&str>,
) -> anyhow::Result<()> {
    let markdown_dir_name = markdown_dir_name.unwrap_or("llms");
    let output_dir = workspace_root.join("web").join("public");
    let book_root = workspace_root.join("book");
    let llms_path = output_dir.join("llms.txt");
    let llms_full_path = output_dir.join("llms-full.txt");
    let llms_markdown_dir = output_dir.join(markdown_dir_name);

    build(
        LlmsConfig::builder()
            .book_root(&book_root)
            .llms_path(&llms_path)
            .llms_full_path(&llms_full_path)
            .llms_markdown_dir(&llms_markdown_dir)
            .base_url(base_url)
            .header(header)
            .markdown_dir_name(markdown_dir_name)
            .build(),
    )
}

struct ChapterInfo {
    name: String,
    path: String,
    content: String,
}

pub fn build(config: LlmsConfig<'_>) -> anyhow::Result<()> {
    println!("Building llms.txt to {}", config.llms_path.display());
    println!(
        "Building llms-full.txt to {}",
        config.llms_full_path.display()
    );
    println!(
        "Building llms Markdown files to {}",
        config.llms_markdown_dir.display()
    );

    let mdbook = MDBook::load(config.book_root)
        .with_context(|| format!("Failed to load book from {}", config.book_root.display()))?;

    let chapters: Vec<ChapterInfo> = mdbook
        .iter()
        .filter_map(|item| match item {
            BookItem::Chapter(chapter) if !chapter.is_draft_chapter() => Some(chapter),
            _ => None,
        })
        .map(|chapter| {
            let path = chapter
                .path
                .as_ref()
                .with_context(|| format!("Missing path for book chapter '{}'", chapter.name))?;

            Ok(ChapterInfo {
                name: chapter.name.clone(),
                path: book_markdown_path(path)?,
                content: chapter.content.clone(),
            })
        })
        .collect::<anyhow::Result<_>>()?;

    let llms_txt = build_llms_txt(
        &chapters,
        config.header,
        config.base_url,
        config.markdown_dir_name,
    );
    let llms_full_txt = build_llms_full_txt(&chapters, config.header);

    ensure_parent_dir(config.llms_path)?;
    ensure_parent_dir(config.llms_full_path)?;
    write_llms_markdown_files(&chapters, config.llms_markdown_dir)?;

    fs::write(config.llms_path, llms_txt)
        .with_context(|| format!("Failed to write llms.txt to {}", config.llms_path.display()))?;

    fs::write(config.llms_full_path, llms_full_txt).with_context(|| {
        format!(
            "Failed to write llms-full.txt to {}",
            config.llms_full_path.display()
        )
    })?;

    println!("llms.txt and llms-full.txt built successfully");
    Ok(())
}

fn ensure_parent_dir(path: &Path) -> anyhow::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create output directory {}", parent.display()))?;
    }
    Ok(())
}

fn book_markdown_path(path: &Path) -> anyhow::Result<String> {
    path.to_slash()
        .map(|path| path.into_owned())
        .with_context(|| format!("Book chapter path is not valid UTF-8: {}", path.display()))
}

fn write_llms_markdown_files(
    chapters: &[ChapterInfo],
    llms_markdown_dir: &Path,
) -> anyhow::Result<()> {
    if llms_markdown_dir.exists() {
        fs::remove_dir_all(llms_markdown_dir).with_context(|| {
            format!(
                "Failed to clear existing llms Markdown directory {}",
                llms_markdown_dir.display()
            )
        })?;
    }

    fs::create_dir_all(llms_markdown_dir).with_context(|| {
        format!(
            "Failed to create llms Markdown directory {}",
            llms_markdown_dir.display()
        )
    })?;

    for chapter in chapters {
        let path = llms_markdown_dir.join(&chapter.path);
        ensure_parent_dir(&path)?;
        fs::write(&path, &chapter.content)
            .with_context(|| format!("Failed to write llms Markdown file {}", path.display()))?;
    }

    Ok(())
}

fn build_llms_txt(
    chapters: &[ChapterInfo],
    header: &str,
    base_url: &str,
    markdown_dir_name: &str,
) -> String {
    let mut output = String::new();
    output.push_str(header);
    output.push_str("\n## Docs\n\n");

    for chapter in chapters {
        let url = format!("{}/{}/{}", base_url, markdown_dir_name, chapter.path);
        output.push_str(&format!("- [{}]({})\n", chapter.name, url));
    }

    output
}

fn build_llms_full_txt(chapters: &[ChapterInfo], header: &str) -> String {
    let mut output = String::new();
    output.push_str(header);
    output.push_str("\n## Docs\n\n");

    for chapter in chapters {
        output.push_str(&chapter.content);
        output.push_str("\n\n---\n\n");
    }

    output
}
