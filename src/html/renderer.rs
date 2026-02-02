use crate::bluesky::types::{Author, Embed, EmbedImage, EmbedRecord, Thread, ThreadPost};
use crate::html::templates::{
    base_template_with_options, render_avatar_html, TemplateOptions, HEADER_TEMPLATE,
};

pub fn render_thread(thread: &Thread) -> String {
    let mut content = String::new();

    content.push_str(&render_header(&thread.author));

    content.push_str("<main class=\"thread\">\n");
    for post in &thread.posts {
        content.push_str(&render_post(post, &thread.author.handle));
    }
    content.push_str("</main>\n");

    content.push_str(&render_footer(thread));

    let title = format!(
        "Thread by @{} - sklonger",
        html_escape::encode_text(&thread.author.handle)
    );

    let options = TemplateOptions {
        favicon_url: thread.author.avatar_url.as_deref(),
        lang: thread.primary_language(),
    };
    base_template_with_options(&title, &content, options)
}

fn render_header(author: &Author) -> String {
    let author_name = author.display_name.as_deref().unwrap_or(&author.handle);
    let avatar = render_avatar_html(author.avatar_url.as_deref(), author_name);
    let profile_url = author.profile_url();

    HEADER_TEMPLATE
        .replace(
            "{profile_url}",
            html_escape::encode_quoted_attribute(&profile_url).as_ref(),
        )
        .replace("{avatar}", &avatar)
        .replace(
            "{display_name}",
            html_escape::encode_text(author_name).as_ref(),
        )
        .replace(
            "{handle}",
            html_escape::encode_text(&author.handle).as_ref(),
        )
}

/// Render a single post as an HTML article element.
/// This is public to support streaming rendering.
pub fn render_post(post: &ThreadPost, author_handle: &str) -> String {
    let text = linkify_text(&html_escape::encode_text(&post.text));
    let embed_html = post.embed.as_ref().map(render_embed).unwrap_or_default();
    let timestamp = post.created_at.format("%b %d, %Y at %H:%M UTC").to_string();

    // Extract post ID from URI (at://did:plc:xxx/app.bsky.feed.post/abc123 -> abc123)
    let post_id = post.uri.rsplit('/').next().unwrap_or("");
    let post_url = format!(
        "https://bsky.app/profile/{}/post/{}",
        author_handle, post_id
    );

    let mut meta_parts = vec![format!(
        r#"<time datetime="{}">{}</time>"#,
        post.created_at.to_rfc3339(),
        timestamp
    )];

    if let Some(likes) = post.like_count.filter(|&n| n > 0) {
        meta_parts.push(format!("{} likes", likes));
    }

    if let Some(reposts) = post.repost_count.filter(|&n| n > 0) {
        meta_parts.push(format!("{} reposts", reposts));
    }

    format!(
        r#"<article class="post">
    <div class="post-text">{text}</div>
    {embed}
    <a href="{post_url}" target="_blank" rel="noopener" class="post-meta">{meta}</a>
</article>
"#,
        text = text,
        embed = embed_html,
        post_url = html_escape::encode_quoted_attribute(&post_url),
        meta = meta_parts.join(" &middot; ")
    )
}

fn render_embed(embed: &Embed) -> String {
    match embed {
        Embed::Images(images) => render_images(images),
        Embed::Video(video) => render_video(video),
        Embed::External(external) => render_external(external),
        Embed::Record(record) => render_record(record),
        Embed::RecordWithMedia { record, media } => {
            format!("{}{}", render_record(record), render_embed(media))
        }
    }
}

fn render_images(images: &[EmbedImage]) -> String {
    if images.is_empty() {
        return String::new();
    }

    let layout = match images.len() {
        1 => "single",
        2 => "double",
        _ => "grid",
    };
    let grid_class = format!("embed-images {}", layout);

    let images_html: String = images
        .iter()
        .map(|img| {
            let aspect_style = img
                .aspect_ratio
                .as_ref()
                .map(|ar| format!("aspect-ratio: {} / {};", ar.width, ar.height))
                .unwrap_or_default();

            format!(
                r#"<a href="{fullsize}" target="_blank" rel="noopener" class="embed-image-link">
    <img src="{thumb}" alt="{alt}" class="embed-image" style="{style}" loading="lazy">
</a>"#,
                fullsize = html_escape::encode_quoted_attribute(&img.fullsize_url),
                thumb = html_escape::encode_quoted_attribute(&img.thumb_url),
                alt = html_escape::encode_quoted_attribute(&img.alt),
                style = aspect_style
            )
        })
        .collect();

    format!(r#"<div class="{grid_class}">{images_html}</div>"#)
}

fn render_video(video: &crate::bluesky::types::EmbedVideo) -> String {
    let aspect_style = video
        .aspect_ratio
        .as_ref()
        .map(|ar| format!("aspect-ratio: {} / {};", ar.width, ar.height))
        .unwrap_or_else(|| String::from("aspect-ratio: 16 / 9;"));

    let alt_text = video.alt.as_deref().unwrap_or("Video");

    // Use HLS.js for video playback since the playlist is m3u8
    format!(
        r#"<div class="embed-video" style="{style}">
    <video controls playsinline preload="metadata" aria-label="{alt}">
        <source src="{playlist}" type="application/x-mpegURL">
        Your browser does not support HLS video.
    </video>
</div>"#,
        style = aspect_style,
        alt = html_escape::encode_quoted_attribute(alt_text),
        playlist = html_escape::encode_quoted_attribute(&video.playlist_url)
    )
}

fn render_external(external: &crate::bluesky::types::EmbedExternal) -> String {
    let thumb_html = external
        .thumb_url
        .as_ref()
        .map(|url| {
            format!(
                r#"<img src="{}" alt="" class="external-thumb" loading="lazy">"#,
                html_escape::encode_quoted_attribute(url)
            )
        })
        .unwrap_or_default();

    format!(
        r#"<a href="{uri}" target="_blank" rel="noopener" class="embed-external">
    {thumb}
    <div class="external-info">
        <div class="external-title">{title}</div>
        <div class="external-description">{description}</div>
    </div>
</a>"#,
        uri = html_escape::encode_quoted_attribute(&external.uri),
        thumb = thumb_html,
        title = html_escape::encode_text(&external.title),
        description = html_escape::encode_text(&external.description)
    )
}

fn render_record(record: &EmbedRecord) -> String {
    let author_name = record
        .author
        .display_name
        .as_deref()
        .unwrap_or(&record.author.handle);
    let avatar_html = render_avatar_html(record.author.avatar_url.as_deref(), author_name);

    // Extract post ID from URI for link
    let post_id = record.uri.rsplit('/').next().unwrap_or("");
    let post_url = format!(
        "https://bsky.app/profile/{}/post/{}",
        record.author.handle, post_id
    );

    let timestamp = record.created_at.format("%b %d, %Y").to_string();

    // Truncate text for preview (max 300 chars)
    let text_preview = if record.text.len() > 300 {
        format!("{}...", &record.text[..297])
    } else {
        record.text.clone()
    };

    // Render any nested embeds (e.g., images in quoted post)
    let nested_embed_html = record
        .embed
        .as_ref()
        .map(|e| render_embed(e))
        .unwrap_or_default();

    format!(
        r#"<a href="{post_url}" target="_blank" rel="noopener" class="embed-record">
    <div class="record-header">
        {avatar}
        <div class="record-author-info">
            <span class="record-author-name">{author_name}</span>
            <span class="record-author-handle">@{handle}</span>
        </div>
    </div>
    <div class="record-text">{text}</div>
    {nested_embed}
    <div class="record-meta">{timestamp}</div>
</a>"#,
        post_url = html_escape::encode_quoted_attribute(&post_url),
        avatar = avatar_html,
        author_name = html_escape::encode_text(author_name),
        handle = html_escape::encode_text(&record.author.handle),
        text = html_escape::encode_text(&text_preview),
        nested_embed = nested_embed_html,
        timestamp = timestamp
    )
}

fn render_footer(thread: &Thread) -> String {
    let original_url = thread
        .original_post_url()
        .unwrap_or_else(|| "https://bsky.app".to_string());

    format!(
        r#"<footer>
    <a href="{url}" target="_blank" rel="noopener">View original on Bluesky</a>
</footer>
"#,
        url = html_escape::encode_quoted_attribute(&original_url)
    )
}

fn linkify_text(text: &str) -> String {
    use std::sync::OnceLock;
    static URL_PATTERN: OnceLock<regex_lite::Regex> = OnceLock::new();
    let pattern = URL_PATTERN
        .get_or_init(|| regex_lite::Regex::new(r"https?://[^\s<>]+").expect("URL regex is valid"));

    pattern
        .replace_all(text, |caps: &regex_lite::Captures| {
            let url = &caps[0];
            let display_url = if url.len() > 40 {
                format!("{}...", &url[..37])
            } else {
                url.to_string()
            };
            format!(
                r#"<a href="{}" target="_blank" rel="noopener">{}</a>"#,
                url, display_url
            )
        })
        .to_string()
}
