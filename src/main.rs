use anyhow::{anyhow, bail, Result};
use clap::Parser;
use std::{fs, path::Path};

mod content;
mod layout;
mod rss;
mod sitemap;
mod templates;

use crate::content::Content;

#[derive(Debug, Parser)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Parser)]
enum Commands {
    #[clap(name = "build")]
    Build,
    #[clap(name = "export-weekly")]
    ExportWeekly { num: u16 },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Build => build(),
        Commands::ExportWeekly { num } => export_weekly(num),
    }
}

fn build() -> Result<()> {
    // Parse content
    let content = Content::parse(fs::read_dir("content")?)?;

    // Recreate dir
    fs::remove_dir_all("dist").ok();
    fs::create_dir_all("dist")?;

    // Copy static files
    copy_dir("static", "dist/")?;

    // Generate CSS
    let sass_options = grass::Options::default().load_path("styles/");
    let css = grass::from_path("styles/main.scss", &sass_options)?;
    fs::write("dist/main.css", css)?;

    // Generate articles
    fs::write("dist/index.html", templates::index(&content)?.into_string())?;
    for article in &content.articles {
        fs::create_dir_all(format!("dist/articles/{}", article.slug))?;
        let path = format!("dist/articles/{}/index.html", article.slug);
        fs::write(&path, templates::article(article)?.into_string())?;
    }

    // Generate weekly
    fs::create_dir_all("dist/weekly")?;
    fs::write(
        "dist/weekly/index.html",
        templates::weekly_index(&content)?.into_string(),
    )?;
    for weekly_issue in &content.weekly {
        fs::create_dir_all(format!("dist/weekly/{}", weekly_issue.num))?;
        let path = format!("dist/weekly/{}/index.html", weekly_issue.num);
        fs::write(&path, templates::weekly(weekly_issue)?.into_string())?;
    }

    // Generate pages
    for page in &content.pages {
        let path = match page.slug.as_str() {
            "404" => "dist/404.html".to_string(),
            _ => {
                fs::create_dir_all(format!("dist/{}", page.slug))?;
                format!("dist/{}/index.html", page.slug)
            }
        };

        fs::write(&path, templates::page(page)?.into_string())?;
    }

    // Generate projects page
    fs::create_dir_all("dist/projects")?;
    fs::write(
        "dist/projects/index.html",
        templates::projects(&content.projects)?.into_string(),
    )?;

    // Generate RSS feeds
    fs::write("dist/feed.xml", rss::render_articles(&content))?;
    fs::write("dist/weekly/feed.xml", rss::render_weekly(&content)?)?;

    // Generate sitemap.xml
    fs::write("dist/sitemap.xml", sitemap::render(&content)?)?;

    Ok(())
}

fn copy_dir<F, T>(from: F, to: T) -> Result<()>
where
    F: AsRef<Path> + Send + Sync,
    T: AsRef<Path> + Send,
{
    // TODO: Turn this into functional code
    let mut dir = fs::read_dir(&from)?;
    while let Some(item) = dir.next().transpose()? {
        let file_name = item.file_name();

        let file_name_str = file_name.to_string_lossy();
        if file_name_str.starts_with('.') && file_name_str != ".well-known" {
            continue;
        }

        let new_path = to.as_ref().join(file_name);
        if new_path.exists() {
            bail!("File or directory already exists: {:?}", new_path)
        }

        if item.path().is_dir() {
            fs::create_dir(&new_path)?;
            copy_dir(item.path(), &new_path)?;
        } else {
            let path = item.path();
            fs::copy(path, new_path)?;
        }
    }

    Ok(())
}

fn export_weekly(num: u16) -> Result<()> {
    let content = Content::parse(fs::read_dir("content")?)?;
    let weekly_issue = content
        .weekly
        .iter()
        .find(|issue| issue.num == num)
        .ok_or_else(|| anyhow!("Weekly issue not found"))?;

    println!("{}", weekly_issue.content);
    println!();

    if let Some(quote_of_the_week) = &weekly_issue.quote_of_the_week {
        println!("## Quote of the Week");
        println!();
        quote_of_the_week.text.split("\n").for_each(|line| {
            println!("> {}", line);
        });
        println!("> — {}", quote_of_the_week.author);
    } else if let Some(toot_of_the_week) = &weekly_issue.toot_of_the_week {
        println!("## Toot of the Week");
        println!();
        toot_of_the_week.text.split("\n").for_each(|line| {
            println!("> {}", line);
        });
        println!(
            "> — [{}]({})",
            toot_of_the_week.author, toot_of_the_week.url
        );
    } else if let Some(tweet_of_the_week) = &weekly_issue.tweet_of_the_week {
        println!("## Tweet of the Week");
        println!();
        tweet_of_the_week.text.split("\n").for_each(|line| {
            println!("> {}", line);
        });
        println!(
            "> — [{}]({})",
            tweet_of_the_week.author, tweet_of_the_week.url
        );
    }
    println!();
    weekly_issue.categories.iter().for_each(|category| {
        println!("## {}", category.title);
        category.stories.iter().for_each(|story| {
            println!("### [{}]({})", story.title, story.url);
            println!(
                "{} min · {}",
                story.reading_time_minutes,
                story.url.host().unwrap()
            );
            println!();
            println!("{}", story.description);
        });
        println!();
    });

    Ok(())
}
