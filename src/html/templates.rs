pub const CSS_STYLES: &str = r#"
:root {
    --bg-primary: #ffffff;
    --bg-secondary: #f8f9fa;
    --text-primary: #1a1a1a;
    --text-secondary: #666666;
    --text-muted: #999999;
    --border-color: #e0e0e0;
    --link-color: #0066cc;
    --accent-color: #0085ff;
}

@media (prefers-color-scheme: dark) {
    :root {
        --bg-primary: #1a1a1a;
        --bg-secondary: #242424;
        --text-primary: #f0f0f0;
        --text-secondary: #b0b0b0;
        --text-muted: #808080;
        --border-color: #404040;
        --link-color: #66b3ff;
        --accent-color: #0085ff;
    }
}

* {
    box-sizing: border-box;
    margin: 0;
    padding: 0;
}

body {
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
    line-height: 1.6;
    color: var(--text-primary);
    background-color: var(--bg-primary);
    max-width: 700px;
    margin: 0 auto;
    padding: 20px;
}

header {
    margin-bottom: 24px;
    padding-bottom: 16px;
    border-bottom: 1px solid var(--border-color);
}

.author {
    display: flex;
    align-items: center;
    gap: 12px;
}

.avatar {
    width: 48px;
    height: 48px;
    border-radius: 50%;
    object-fit: cover;
    background-color: var(--bg-secondary);
}

.avatar-placeholder {
    width: 48px;
    height: 48px;
    border-radius: 50%;
    background-color: var(--accent-color);
    display: flex;
    align-items: center;
    justify-content: center;
    color: white;
    font-weight: bold;
    font-size: 18px;
}

.author-info {
    display: flex;
    flex-direction: column;
}

.display-name {
    font-weight: 600;
    font-size: 16px;
}

.handle {
    color: var(--text-secondary);
    font-size: 14px;
}

.thread {
    display: flex;
    flex-direction: column;
    gap: 0;
}

.post {
    padding: 8px 0;
}

.post-text {
    font-size: 16px;
    white-space: pre-wrap;
    word-wrap: break-word;
    line-height: 1.7;
}

.post-text a {
    color: var(--link-color);
    text-decoration: none;
}

.post-text a:hover {
    text-decoration: underline;
}

.post-meta {
    margin-top: 8px;
    font-size: 11px;
    color: var(--text-muted);
    display: flex;
    gap: 12px;
    opacity: 0.7;
}

.post-meta time {
    color: var(--text-muted);
}

/* Embed styles */
.embed-images {
    margin-top: 12px;
    border-radius: 12px;
    overflow: hidden;
}

.embed-images.single .embed-image-link {
    display: block;
}

.embed-images.double {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 2px;
}

.embed-images.grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 2px;
}

.embed-image-link {
    display: block;
    line-height: 0;
}

.embed-image {
    width: 100%;
    height: auto;
    max-width: 75dvw;
    max-height: 75dvh;
    object-fit: cover;
    background-color: var(--bg-secondary);
}

.embed-images.single .embed-image {
    max-width: 75dvw;
    max-height: 75dvh;
    object-fit: contain;
}

.embed-video {
    margin-top: 12px;
    border-radius: 12px;
    overflow: hidden;
    background-color: var(--bg-secondary);
    max-width: 75dvw;
    max-height: 75dvh;
}

.embed-video video {
    width: 100%;
    height: 100%;
    max-width: 75dvw;
    max-height: 75dvh;
    display: block;
}

.embed-external {
    display: flex;
    margin-top: 12px;
    border: 1px solid var(--border-color);
    border-radius: 12px;
    overflow: hidden;
    text-decoration: none;
    color: inherit;
    transition: background-color 0.15s ease;
}

.embed-external:hover {
    background-color: var(--bg-secondary);
}

.external-thumb {
    width: 120px;
    height: 80px;
    object-fit: cover;
    flex-shrink: 0;
}

.external-info {
    padding: 10px 14px;
    min-width: 0;
}

.external-title {
    font-size: 14px;
    font-weight: 500;
    line-height: 1.3;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
}

.external-description {
    font-size: 12px;
    color: var(--text-secondary);
    margin-top: 2px;
    overflow: hidden;
    text-overflow: ellipsis;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
}

footer {
    margin-top: 24px;
    padding-top: 16px;
    border-top: 1px solid var(--border-color);
    font-size: 13px;
    color: var(--text-muted);
    text-align: center;
}

footer a {
    color: var(--link-color);
    text-decoration: none;
}

footer a:hover {
    text-decoration: underline;
}

.error-page {
    text-align: center;
    padding: 60px 20px;
}

.error-page h1 {
    font-size: 48px;
    margin-bottom: 16px;
    color: var(--text-secondary);
}

.error-page p {
    font-size: 18px;
    margin-bottom: 24px;
    color: var(--text-secondary);
}

.error-page a {
    color: var(--link-color);
    text-decoration: none;
    font-size: 16px;
}

.error-page a:hover {
    text-decoration: underline;
}

@media print {
    body {
        max-width: none;
        padding: 0;
    }

    header, footer {
        border: none;
    }

    .embed-video {
        display: none;
    }
}
"#;

pub fn base_template(title: &str, content: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>{title}</title>
    <style>{css}</style>
</head>
<body>
{content}
</body>
</html>"#,
        title = html_escape::encode_text(title),
        css = CSS_STYLES,
        content = content
    )
}

pub fn error_page(status_code: u16, title: &str, message: &str) -> String {
    let content = format!(
        r#"<main class="error-page">
    <h1>{status_code}</h1>
    <p>{title}: {message}</p>
    <a href="/">Try another thread</a>
</main>"#,
        status_code = status_code,
        title = html_escape::encode_text(title),
        message = html_escape::encode_text(message)
    );
    base_template(&format!("{} - {}", status_code, title), &content)
}
