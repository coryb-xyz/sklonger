use crate::bluesky::types::{Author, Embed, EmbedImage, Thread, ThreadPost};
use crate::html::templates::base_template;

pub fn render_thread(thread: &Thread) -> String {
    let mut content = String::new();

    content.push_str(&render_header(&thread.author));

    content.push_str("<main class=\"thread\">\n");
    for post in &thread.posts {
        content.push_str(&render_post(post));
    }
    content.push_str("</main>\n");

    content.push_str(&render_footer(thread));

    let title = format!(
        "Thread by @{} - skeet-longer",
        html_escape::encode_text(&thread.author.handle)
    );

    base_template(&title, &content)
}

fn render_header(author: &Author) -> String {
    let avatar = match &author.avatar_url {
        Some(url) => format!(
            r#"<img class="avatar" src="{}" alt="Avatar">"#,
            html_escape::encode_quoted_attribute(url)
        ),
        None => {
            let initial = author
                .display_name
                .as_ref()
                .or(Some(&author.handle))
                .and_then(|s| s.chars().next())
                .unwrap_or('?')
                .to_uppercase()
                .to_string();
            format!(r#"<div class="avatar-placeholder">{}</div>"#, initial)
        }
    };

    let display_name = author
        .display_name
        .as_ref()
        .map(|n| html_escape::encode_text(n).to_string())
        .unwrap_or_else(|| html_escape::encode_text(&author.handle).to_string());

    format!(
        r#"<header>
    <div class="author">
        {avatar}
        <div class="author-info">
            <span class="display-name">{display_name}</span>
            <span class="handle">@{handle}</span>
        </div>
    </div>
</header>
"#,
        avatar = avatar,
        display_name = display_name,
        handle = html_escape::encode_text(&author.handle)
    )
}

fn render_post(post: &ThreadPost) -> String {
    let text = linkify_text(&html_escape::encode_text(&post.text));

    let embed_html = post
        .embed
        .as_ref()
        .map(render_embed)
        .unwrap_or_default();

    let timestamp = post.created_at.format("%b %d, %Y at %H:%M UTC").to_string();

    let mut meta_parts = vec![format!(
        r#"<time datetime="{}">{}</time>"#,
        post.created_at.to_rfc3339(),
        timestamp
    )];

    if let Some(likes) = post.like_count {
        if likes > 0 {
            meta_parts.push(format!("{} likes", likes));
        }
    }

    if let Some(reposts) = post.repost_count {
        if reposts > 0 {
            meta_parts.push(format!("{} reposts", reposts));
        }
    }

    format!(
        r#"<article class="post">
    <div class="post-text">{text}</div>
    {embed}
    <div class="post-meta">{meta}</div>
</article>
"#,
        text = text,
        embed = embed_html,
        meta = meta_parts.join(" &middot; ")
    )
}

fn render_embed(embed: &Embed) -> String {
    match embed {
        Embed::Images(images) => render_images(images),
        Embed::Video(video) => render_video(video),
        Embed::External(external) => render_external(external),
    }
}

fn render_images(images: &[EmbedImage]) -> String {
    if images.is_empty() {
        return String::new();
    }

    let grid_class = match images.len() {
        1 => "embed-images single",
        2 => "embed-images double",
        _ => "embed-images grid",
    };

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

    format!(r#"<div class="{}">{}</div>"#, grid_class, images_html)
}

fn render_video(video: &crate::bluesky::types::EmbedVideo) -> String {
    let aspect_style = video
        .aspect_ratio
        .as_ref()
        .map(|ar| format!("aspect-ratio: {} / {};", ar.width, ar.height))
        .unwrap_or_else(|| "aspect-ratio: 16 / 9;".to_string());

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
    let url_pattern = regex_lite::Regex::new(r"https?://[^\s<>]+")
        .unwrap_or_else(|_| regex_lite::Regex::new(r"^$").unwrap());

    url_pattern
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
