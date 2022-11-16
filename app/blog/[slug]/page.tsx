import {
  buildAbsolutePath,
  parseMarkdown,
  renderMarkdown,
} from "../../../lib/markdown";
import { DateTime } from "luxon";
import { promises as fs } from "node:fs";

export async function generateStaticParams() {
  const path = buildAbsolutePath("content/blog");
  const files = await fs.readdir(path);
  return files
    .filter((filename) => filename != "_index.md" && filename.endsWith(".md"))
    .map((filename) => ({
      slug: filename.substring(0, filename.length - 3),
    }));
}

export default async function Blogpost({
  params: { slug },
}: {
  params: { slug: string };
}) {
  const { frontmatter, html, readingTimeMinutes } = await parseMarkdown(
    "content/blog/" + slug + ".md"
  );

  let coverImage = null;
  if (frontmatter.coverImage) {
    const captionHtml = await renderMarkdown(frontmatter.coverImage.caption);
    coverImage = (
      <figure className="cover-image has-caption">
        <img
          src={frontmatter.coverImage.src}
          alt={frontmatter.coverImage.alt}
        />
        <figcaption dangerouslySetInnerHTML={{ __html: captionHtml }} />
      </figure>
    );
  }

  return (
    <article>
      <h1>{frontmatter.title}</h1>
      <span className="details">
        <time dateTime={frontmatter.date}>
          {DateTime.fromISO(frontmatter.date).toFormat("LLL dd, yyyy")}
        </time>{" "}
        &middot; {readingTimeMinutes} min
      </span>
      {coverImage}
      <div dangerouslySetInnerHTML={{ __html: html.toString() }} />
    </article>
  );
}
