use crate::bluesky::types::{Author, Thread, ThreadPost};
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
    <div class="post-meta">{meta}</div>
</article>
"#,
        text = text,
        meta = meta_parts.join(" &middot; ")
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
