pub const CSS_STYLES: &str = r#"
/* Light mode (default) - optimized for readability
   Uses soft white background to reduce harshness
   Text uses dark gray for better contrast without pure black
   WCAG AA compliant contrast ratios */
:root {
    --bg-primary: #fafafa;
    --bg-secondary: #f0f1f2;
    --text-primary: #1f2937;
    --text-secondary: #4b5563;
    --text-muted: #6b7280;
    --border-color: #d1d5db;
    --link-color: #2563eb;
    --accent-color: #3b82f6;
    --toggle-bg: #e5e7eb;
    --toggle-knob: #ffffff;
    --toggle-icon: #f59e0b;
}

/* Dark mode - optimized for reduced eye strain
   Uses dark gray (#121212 base) instead of pure black
   Text uses light gray instead of pure white to reduce halation
   Desaturated accent colors to avoid harshness */
@media (prefers-color-scheme: dark) {
    :root:not([data-theme="light"]) {
        --bg-primary: #121212;
        --bg-secondary: #1e1e1e;
        --text-primary: #e4e4e7;
        --text-secondary: #a1a1aa;
        --text-muted: #71717a;
        --border-color: #3f3f46;
        --link-color: #60a5fa;
        --accent-color: #3b82f6;
        --toggle-bg: #3f3f46;
        --toggle-knob: #18181b;
        --toggle-icon: #fbbf24;
    }
}

/* Explicit dark theme override */
[data-theme="dark"] {
    --bg-primary: #121212;
    --bg-secondary: #1e1e1e;
    --text-primary: #e4e4e7;
    --text-secondary: #a1a1aa;
    --text-muted: #71717a;
    --border-color: #3f3f46;
    --link-color: #60a5fa;
    --accent-color: #3b82f6;
    --toggle-bg: #3f3f46;
    --toggle-knob: #18181b;
    --toggle-icon: #fbbf24;
}

/* Explicit light theme override */
[data-theme="light"] {
    --bg-primary: #fafafa;
    --bg-secondary: #f0f1f2;
    --text-primary: #1f2937;
    --text-secondary: #4b5563;
    --text-muted: #6b7280;
    --border-color: #d1d5db;
    --link-color: #2563eb;
    --accent-color: #3b82f6;
    --toggle-bg: #e5e7eb;
    --toggle-knob: #ffffff;
    --toggle-icon: #f59e0b;
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
    padding-top: 80px;
}

header {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    max-width: 700px;
    margin: 0 auto;
    padding: 16px 20px;
    background-color: var(--bg-primary);
    border-bottom: 1px solid var(--border-color);
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    z-index: 100;
    transition: padding 0.2s ease, transform 0.2s ease;
}

header.compact {
    padding: 8px 20px;
}

header.compact .avatar,
header.compact .avatar-placeholder {
    width: 32px;
    height: 32px;
    font-size: 14px;
}

header.compact .display-name {
    font-size: 14px;
}

header.compact .handle {
    font-size: 12px;
}

a.author {
    display: flex;
    align-items: center;
    gap: 12px;
    min-width: 0;
    text-decoration: none;
    color: inherit;
}

a.author:hover .display-name {
    text-decoration: underline;
}

.avatar {
    width: 48px;
    height: 48px;
    border-radius: 50%;
    object-fit: cover;
    background-color: var(--bg-secondary);
    transition: width 0.2s ease, height 0.2s ease;
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
    transition: width 0.2s ease, height 0.2s ease, font-size 0.2s ease;
}

.author-info {
    display: flex;
    flex-direction: column;
}

.display-name {
    font-weight: 600;
    font-size: 16px;
    transition: font-size 0.2s ease;
}

.handle {
    color: var(--text-secondary);
    transition: font-size 0.2s ease;
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
    text-decoration: none;
    transition: opacity 0.15s ease;
}

.post-meta:hover {
    opacity: 1;
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

/* Landing page styles */
.landing {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    min-height: calc(100vh - 200px);
    text-align: center;
    padding: 40px 20px;
}

.landing-title {
    font-size: 3rem;
    font-weight: 700;
    margin-bottom: 32px;
    color: var(--text-primary);
}

.landing-title .small {
    font-size: 0.25em;
    vertical-align: middle;
}

.landing-form {
    display: flex;
    gap: 8px;
    width: 100%;
    max-width: 500px;
    margin-bottom: 24px;
}

.landing-input {
    flex: 1;
    padding: 12px 16px;
    font-size: 16px;
    border: 1px solid var(--border-color);
    border-radius: 8px;
    background-color: var(--bg-secondary);
    color: var(--text-primary);
    outline: none;
    transition: border-color 0.15s ease;
}

.landing-input::placeholder {
    color: var(--text-muted);
}

.landing-input:focus {
    border-color: var(--accent-color);
}

.landing-button {
    padding: 12px 24px;
    font-size: 16px;
    font-weight: 500;
    border: none;
    border-radius: 8px;
    background-color: var(--accent-color);
    color: white;
    cursor: pointer;
    transition: opacity 0.15s ease;
}

.landing-button:hover {
    opacity: 0.9;
}

.landing-button:focus {
    outline: 2px solid var(--accent-color);
    outline-offset: 2px;
}

.landing-description {
    font-size: 16px;
    color: var(--text-secondary);
    max-width: 400px;
}

.landing-header {
    position: fixed;
    top: 0;
    right: 0;
    padding: 16px 20px;
    z-index: 100;
}

.home-link {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 36px;
    height: 36px;
    border-radius: 50%;
    background-color: var(--bg-secondary);
    color: var(--text-secondary);
    text-decoration: none;
    transition: background-color 0.15s ease, color 0.15s ease;
    flex-shrink: 0;
}

.home-link:hover {
    background-color: var(--border-color);
    color: var(--text-primary);
}

.home-link svg {
    width: 18px;
    height: 18px;
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

/* Theme toggle switch */
.theme-toggle {
    position: relative;
    width: 52px;
    height: 28px;
    background-color: var(--toggle-bg);
    border-radius: 14px;
    cursor: pointer;
    transition: background-color 0.2s ease;
    border: none;
    padding: 0;
    flex-shrink: 0;
}

.theme-toggle:focus {
    outline: 2px solid var(--accent-color);
    outline-offset: 2px;
}

.theme-toggle-knob {
    position: absolute;
    top: 3px;
    left: 3px;
    width: 22px;
    height: 22px;
    background-color: var(--toggle-knob);
    border-radius: 50%;
    transition: transform 0.2s ease;
    display: flex;
    align-items: center;
    justify-content: center;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.2);
}

.theme-toggle-knob svg {
    width: 14px;
    height: 14px;
    color: var(--toggle-icon);
}

[data-theme="dark"] .theme-toggle-knob,
:root:not([data-theme="light"]) .theme-toggle-knob {
    transform: translateX(24px);
}

@media (prefers-color-scheme: light) {
    :root:not([data-theme="dark"]) .theme-toggle-knob {
        transform: translateX(0);
    }
}

@media (prefers-color-scheme: dark) {
    :root:not([data-theme="light"]) .theme-toggle-knob {
        transform: translateX(24px);
    }
}

/* Sun icon for light mode */
.icon-sun {
    display: block;
}

.icon-moon {
    display: none;
}

[data-theme="dark"] .icon-sun {
    display: none;
}

[data-theme="dark"] .icon-moon {
    display: block;
}

@media (prefers-color-scheme: dark) {
    :root:not([data-theme="light"]) .icon-sun {
        display: none;
    }
    :root:not([data-theme="light"]) .icon-moon {
        display: block;
    }
}

@media (prefers-color-scheme: light) {
    :root:not([data-theme="dark"]) .icon-sun {
        display: block;
    }
    :root:not([data-theme="dark"]) .icon-moon {
        display: none;
    }
}

[data-theme="light"] .icon-sun {
    display: block;
}

[data-theme="light"] .icon-moon {
    display: none;
}

/* Mobile responsive adjustments */
@media (max-width: 600px) {
    body {
        padding-top: 70px;
    }

    header {
        padding: 12px 16px;
    }

    header.compact {
        padding: 8px 16px;
    }

    .avatar {
        width: 40px;
        height: 40px;
    }

    .avatar-placeholder {
        width: 40px;
        height: 40px;
        font-size: 16px;
    }

    a.author {
        gap: 10px;
    }

    .author-info {
        max-width: 120px;
        overflow: hidden;
    }

    .display-name {
        font-size: 14px;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .handle {
        font-size: 12px;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .home-link {
        width: 32px;
        height: 32px;
    }

    .home-link svg {
        width: 16px;
        height: 16px;
    }

    .theme-toggle {
        width: 44px;
        height: 24px;
        border-radius: 12px;
    }

    .theme-toggle-knob {
        width: 18px;
        height: 18px;
    }

    .theme-toggle-knob svg {
        width: 12px;
        height: 12px;
    }

    [data-theme="dark"] .theme-toggle-knob,
    :root:not([data-theme="light"]) .theme-toggle-knob {
        transform: translateX(20px);
    }

    header.compact .avatar,
    header.compact .avatar-placeholder {
        width: 28px;
        height: 28px;
        font-size: 12px;
    }

    header.compact .display-name {
        font-size: 12px;
    }

    header.compact .handle {
        font-size: 10px;
    }
}

@media print {
    body {
        max-width: none;
        padding: 0;
        padding-top: 0;
    }

    header {
        position: static;
        border: none;
    }

    footer {
        border: none;
    }

    .embed-video {
        display: none;
    }

    .theme-toggle {
        display: none;
    }
}
"#;

const THEME_SCRIPT: &str = r#"
(function() {
    var stored = localStorage.getItem('theme');
    if (stored) {
        document.documentElement.setAttribute('data-theme', stored);
    }
})();
"#;

const THEME_TOGGLE_SCRIPT: &str = r#"
(function() {
    var toggle = document.getElementById('theme-toggle');
    if (!toggle) return;

    function getEffectiveTheme() {
        var stored = localStorage.getItem('theme');
        if (stored) return stored;
        return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
    }

    function updateAriaChecked() {
        var isDark = getEffectiveTheme() === 'dark';
        toggle.setAttribute('aria-checked', isDark ? 'true' : 'false');
    }

    // Set initial aria-checked state
    updateAriaChecked();

    toggle.addEventListener('click', function() {
        var current = getEffectiveTheme();
        var next = current === 'dark' ? 'light' : 'dark';
        document.documentElement.setAttribute('data-theme', next);
        localStorage.setItem('theme', next);
        updateAriaChecked();
    });

    toggle.addEventListener('keydown', function(e) {
        if (e.key === 'Enter' || e.key === ' ') {
            e.preventDefault();
            toggle.click();
        }
    });
})();

// Sticky header compact mode on scroll
(function() {
    var header = document.querySelector('header');
    if (!header) return;

    var scrollThreshold = 50;
    var isCompact = false;

    function updateHeaderState() {
        var shouldBeCompact = window.scrollY > scrollThreshold;
        if (shouldBeCompact !== isCompact) {
            isCompact = shouldBeCompact;
            if (isCompact) {
                header.classList.add('compact');
            } else {
                header.classList.remove('compact');
            }
        }
    }

    window.addEventListener('scroll', updateHeaderState, { passive: true });
    updateHeaderState();
})();
"#;

/// Template options for customizing the HTML output
#[derive(Default)]
pub struct TemplateOptions<'a> {
    pub favicon_url: Option<&'a str>,
    pub lang: Option<&'a str>,
}

pub fn base_template(title: &str, content: &str) -> String {
    base_template_with_options(title, content, TemplateOptions::default())
}

pub fn base_template_with_options(title: &str, content: &str, options: TemplateOptions) -> String {
    let favicon_tag = options
        .favicon_url
        .map(|url| {
            format!(
                r#"<link rel="icon" type="image/png" href="{}">"#,
                html_escape::encode_quoted_attribute(url)
            )
        })
        .unwrap_or_default();

    // Use provided language or default to "en"
    let lang = options.lang.unwrap_or("en");

    format!(
        r#"<!DOCTYPE html>
<html lang="{lang}">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>{title}</title>
    {favicon}
    <script>{theme_init}</script>
    <style>{css}</style>
</head>
<body>
{content}
<script>{theme_toggle}</script>
</body>
</html>"#,
        lang = html_escape::encode_quoted_attribute(lang),
        title = html_escape::encode_text(title),
        favicon = favicon_tag,
        theme_init = THEME_SCRIPT,
        css = CSS_STYLES,
        content = content,
        theme_toggle = THEME_TOGGLE_SCRIPT
    )
}

pub fn landing_page() -> String {
    let content = r#"<div class="landing-header">
    <button id="theme-toggle" class="theme-toggle" role="switch" aria-checked="false" aria-label="Toggle dark mode">
        <span class="theme-toggle-knob">
            <svg class="icon-sun" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <circle cx="12" cy="12" r="5"></circle>
                <line x1="12" y1="1" x2="12" y2="3"></line>
                <line x1="12" y1="21" x2="12" y2="23"></line>
                <line x1="4.22" y1="4.22" x2="5.64" y2="5.64"></line>
                <line x1="18.36" y1="18.36" x2="19.78" y2="19.78"></line>
                <line x1="1" y1="12" x2="3" y2="12"></line>
                <line x1="21" y1="12" x2="23" y2="12"></line>
                <line x1="4.22" y1="19.78" x2="5.64" y2="18.36"></line>
                <line x1="18.36" y1="5.64" x2="19.78" y2="4.22"></line>
            </svg>
            <svg class="icon-moon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"></path>
            </svg>
        </span>
    </button>
</div>
<main class="landing">
    <h1 class="landing-title">SK<span class="small">eet</span> LONGER</h1>
    <form class="landing-form" action="/" method="get">
        <input type="url" name="url" class="landing-input" placeholder="https://bsky.app/profile/.../post/..." required>
        <button type="submit" class="landing-button">Go</button>
    </form>
    <p class="landing-description">Paste a Bluesky post URL to view the full thread as a single, readable page.</p>
</main>"#;
    base_template("Skeet Longer - Bluesky Thread Reader", content)
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
