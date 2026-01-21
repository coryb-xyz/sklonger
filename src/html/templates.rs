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
    padding: 16px 0;
    border-bottom: 1px solid var(--border-color);
}

.post:last-child {
    border-bottom: none;
}

.post-text {
    font-size: 15px;
    white-space: pre-wrap;
    word-wrap: break-word;
}

.post-text a {
    color: var(--link-color);
    text-decoration: none;
}

.post-text a:hover {
    text-decoration: underline;
}

.post-meta {
    margin-top: 12px;
    font-size: 13px;
    color: var(--text-muted);
    display: flex;
    gap: 16px;
}

.post-meta time {
    color: var(--text-secondary);
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
