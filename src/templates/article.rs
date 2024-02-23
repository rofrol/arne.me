use anyhow::Result;
use maud::{html, PreEscaped};
use url::Url;

use crate::{
    content::{Article, Content},
    templates::{
        format_date,
        layout::{Context, Head, OgType},
    },
};

pub fn render(article: &Article) -> Result<Context> {
    Ok(Context::new(
        Head {
            title: article.title.clone(),
            description: article.description.clone(),
            url: Url::parse(&format!("https://arne.me/articles/{}", article.slug))?,
            og_type: OgType::Article,
        },
        html! {
            article.article {
                header.article__header {
                    h1 { (article.title) }
                    em.article__byline {
                        time datetime=(article.published.format("%Y-%m-%d")) { (format_date(article.published)) }
                        (PreEscaped(" &middot; "))
                        (article.location)
                        @if article.hackernews.is_some() || article.lobsters.is_some() {
                            (PreEscaped(" &middot; "))
                            @if let Some(hackernews) = &article.hackernews {
                                a href=(hackernews) { "HN" }
                                @if article.lobsters.is_some() {
                                    ", "
                                }
                            }
                            @if let Some(lobsters) = &article.lobsters {
                                a href=(lobsters) { "Lobsters" }
                            }
                        }

                    }
                }
                (PreEscaped(article.content_html.clone()))
            }
        },
    ))
}

pub fn render_index(content: &Content) -> Result<Context> {
    Ok(Context::new(
        Head {
            title: "Articles".to_string(),
            description: "Articles by Arne Bahlo.".to_string(),
            url: Url::parse("https://arne.me/articles")?,
            og_type: OgType::Website,
        },
        html! {
            section.page {
                h1 { "Articles" }
                @for article in content.articles.iter().filter(|a| !a.hidden) {
                    div {
                        h3.inheritFontSize { a href=(format!("/articles/{}", article.slug)) { (article.title) } }
                        em.article__byline {
                            time datetime=(article.published.format("%Y-%m-%d")) {(format_date(article.published))}
                            (PreEscaped(" &middot; "))
                            (article.location)
                        }
                    }
                }
            }
        },
    ))
}
