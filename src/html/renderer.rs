use crate::bluesky::types::{Author, Embed, EmbedImage, Thread, ThreadPost};
use crate::html::templates::{base_template_with_options, TemplateOptions};

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
        "Thread by @{} - skeet-longer",
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

    let avatar = match &author.avatar_url {
        Some(url) => format!(
            r#"<img class="avatar" src="{}" alt="{}'s avatar">"#,
            html_escape::encode_quoted_attribute(url),
            html_escape::encode_quoted_attribute(author_name)
        ),
        None => {
            let initial = author_name
                .chars()
                .next()
                .unwrap_or('?')
                .to_uppercase()
                .to_string();
            format!(
                r#"<div class="avatar-placeholder" role="img" aria-label="{}'s avatar">{}</div>"#,
                html_escape::encode_quoted_attribute(author_name),
                initial
            )
        }
    };

    let display_name = html_escape::encode_text(author_name).to_string();

    let profile_url = author.profile_url();

    format!(
        r#"<header>
    <a href="/" class="home-link" aria-label="Home">
        <svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
            <path d="M11.47 3.84a.75.75 0 011.06 0l8.69 8.69a.75.75 0 101.06-1.06l-8.689-8.69a2.25 2.25 0 00-3.182 0l-8.69 8.69a.75.75 0 001.061 1.06l8.69-8.69z" />
            <path d="M12 5.432l8.159 8.159c.03.03.06.058.091.086v6.198c0 1.035-.84 1.875-1.875 1.875H15a.75.75 0 01-.75-.75v-4.5a.75.75 0 00-.75-.75h-3a.75.75 0 00-.75.75V21a.75.75 0 01-.75.75H5.625a1.875 1.875 0 01-1.875-1.875v-6.198a2.29 2.29 0 00.091-.086L12 5.43z" />
        </svg>
    </a>
    <a href="{profile_url}" class="author" target="_blank" rel="noopener">
        {avatar}
        <div class="author-info">
            <span class="display-name">{display_name}</span>
            <span class="handle">@{handle}</span>
        </div>
    </a>
    <button id="theme-toggle" class="theme-toggle" type="button" aria-label="Toggle dark mode" role="switch" aria-checked="false">
        <span class="theme-toggle-knob" aria-hidden="true">
            <svg class="icon-sun" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
                <path d="M12 2.25a.75.75 0 01.75.75v2.25a.75.75 0 01-1.5 0V3a.75.75 0 01.75-.75zM7.5 12a4.5 4.5 0 119 0 4.5 4.5 0 01-9 0zM18.894 6.166a.75.75 0 00-1.06-1.06l-1.591 1.59a.75.75 0 101.06 1.061l1.591-1.59zM21.75 12a.75.75 0 01-.75.75h-2.25a.75.75 0 010-1.5H21a.75.75 0 01.75.75zM17.834 18.894a.75.75 0 001.06-1.06l-1.59-1.591a.75.75 0 10-1.061 1.06l1.59 1.591zM12 18a.75.75 0 01.75.75V21a.75.75 0 01-1.5 0v-2.25A.75.75 0 0112 18zM7.758 17.303a.75.75 0 00-1.061-1.06l-1.591 1.59a.75.75 0 001.06 1.061l1.591-1.59zM6 12a.75.75 0 01-.75.75H3a.75.75 0 010-1.5h2.25A.75.75 0 016 12zM6.697 7.757a.75.75 0 001.06-1.06l-1.59-1.591a.75.75 0 00-1.061 1.06l1.59 1.591z" />
            </svg>
            <svg class="icon-moon" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
                <path fill-rule="evenodd" d="M9.528 1.718a.75.75 0 01.162.819A8.97 8.97 0 009 6a9 9 0 009 9 8.97 8.97 0 003.463-.69.75.75 0 01.981.98 10.503 10.503 0 01-9.694 6.46c-5.799 0-10.5-4.701-10.5-10.5 0-4.368 2.667-8.112 6.46-9.694a.75.75 0 01.818.162z" clip-rule="evenodd" />
            </svg>
        </span>
    </button>
</header>
"#,
        profile_url = html_escape::encode_quoted_attribute(&profile_url),
        avatar = avatar,
        display_name = display_name,
        handle = html_escape::encode_text(&author.handle)
    )
}

fn render_post(post: &ThreadPost, author_handle: &str) -> String {
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
