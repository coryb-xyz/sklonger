// Static assets loaded from external files at compile time
pub const CSS_STYLES: &str = include_str!("templates/styles.css");
pub const HEADER_TEMPLATE: &str = include_str!("templates/header.html");

const THEME_SCRIPT: &str = include_str!("templates/theme-init.js");
const THEME_TOGGLE_SCRIPT: &str = include_str!("templates/theme-toggle.js");
const PWA_META_TAGS: &str = include_str!("templates/pwa-meta.html");
const SERVICE_WORKER_REGISTRATION: &str = include_str!("templates/sw-register.js");
const PWA_INSTALL_SCRIPT: &str = include_str!("templates/pwa-install.js");
const GALLERY_SCRIPT: &str = include_str!("templates/gallery.js");

/// Social media meta tags for Open Graph and Twitter Cards
#[derive(Default)]
pub struct SocialMeta<'a> {
    /// Page title for og:title and twitter:title
    pub title: Option<&'a str>,
    /// Description for og:description and twitter:description (will be truncated to 160 chars)
    pub description: Option<&'a str>,
    /// Canonical URL for og:url
    pub url: Option<&'a str>,
    /// Image URL for og:image and twitter:image
    pub image_url: Option<&'a str>,
    /// Content type (e.g., "article", "website")
    pub og_type: Option<&'a str>,
}

/// Template options for customizing the HTML output
#[derive(Default)]
pub struct TemplateOptions<'a> {
    pub favicon_url: Option<&'a str>,
    pub lang: Option<&'a str>,
    pub social: Option<SocialMeta<'a>>,
}

/// Generate a favicon link tag from an optional URL.
fn render_favicon_tag(url: Option<&str>) -> String {
    url.map(|u| {
        format!(
            r#"<link rel="icon" type="image/png" href="{}">"#,
            html_escape::encode_quoted_attribute(u)
        )
    })
    .unwrap_or_default()
}

/// Render avatar HTML - either an img tag for avatars or a placeholder div with initials.
pub fn render_avatar_html(avatar_url: Option<&str>, author_name: &str) -> String {
    match avatar_url {
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
    }
}

/// Configuration for client-side polling of thread updates
#[derive(Debug, Clone)]
pub struct PollingConfig {
    pub handle: String,
    pub post_id: String,
    pub last_cid: String,
    pub initial_interval: u64,
    pub max_interval: u64,
    pub disable_after: u64,
}

/// Generate the polling script with the given configuration
fn render_poll_script(config: &PollingConfig) -> String {
    format!(
        r#"
(function() {{
    var cfg = {{
        handle: '{handle}',
        postId: '{post_id}',
        lastCid: '{last_cid}',
        interval: {initial_interval} * 1000,
        maxInterval: {max_interval} * 1000,
        disableAfter: {disable_after} * 1000,
        initialInterval: {initial_interval} * 1000
    }};

    var noUpdateSince = Date.now();
    var lastPollTime = Date.now();
    var timerId = null;
    var stopped = false;
    var refreshBtn = null;

    // Find and initialize the refresh button in header
    function initRefreshButton() {{
        refreshBtn = document.getElementById('refresh-btn');
        if (!refreshBtn) return;

        refreshBtn.addEventListener('click', function() {{
            if (stopped) {{
                stopped = false;
                noUpdateSince = Date.now();
            }}
            refreshBtn.disabled = true;
            refreshBtn.classList.add('spinning');

            if (timerId) clearTimeout(timerId);

            var url = '/api/thread/updates?handle=' + encodeURIComponent(cfg.handle) +
                      '&post_id=' + encodeURIComponent(cfg.postId) +
                      '&since_cid=' + encodeURIComponent(cfg.lastCid);

            fetch(url)
                .then(function(r) {{
                    if (r.status === 204) return null;
                    if (!r.ok) throw new Error('Refresh failed: ' + r.status);
                    cfg.lastCid = r.headers.get('X-Last-CID') || cfg.lastCid;
                    return r.text();
                }})
                .then(function(html) {{
                    if (html) {{
                        insertPosts(html);
                        noUpdateSince = Date.now();
                        cfg.interval = cfg.initialInterval;
                    }}
                    refreshBtn.classList.remove('spinning');
                    refreshBtn.disabled = false;
                    schedule();
                }})
                .catch(function(e) {{
                    console.error('Refresh error:', e);
                    refreshBtn.classList.remove('spinning');
                    refreshBtn.disabled = false;
                    schedule();
                }});
        }});
    }}

    function poll() {{
        if (stopped) return;

        var now = Date.now();
        if (now - noUpdateSince > cfg.disableAfter) {{
            stopped = true;
            return;
        }}

        lastPollTime = now;

        var url = '/api/thread/updates?handle=' + encodeURIComponent(cfg.handle) +
                  '&post_id=' + encodeURIComponent(cfg.postId) +
                  '&since_cid=' + encodeURIComponent(cfg.lastCid);

        fetch(url)
            .then(function(r) {{
                if (r.status === 204) {{
                    cfg.interval = Math.min(cfg.interval * 1.5, cfg.maxInterval);
                    return null;
                }}
                if (!r.ok) throw new Error('Poll failed: ' + r.status);
                cfg.lastCid = r.headers.get('X-Last-CID') || cfg.lastCid;
                return r.text();
            }})
            .then(function(html) {{
                if (html) {{
                    insertPosts(html);
                    noUpdateSince = Date.now();
                    cfg.interval = cfg.initialInterval;
                }}
                schedule();
            }})
            .catch(function(e) {{
                console.error('Poll error:', e);
                cfg.interval = Math.min(cfg.interval * 2, cfg.maxInterval);
                schedule();
            }});
    }}

    function schedule() {{
        if (stopped) return;
        timerId = setTimeout(poll, cfg.interval);
    }}

    function insertPosts(html) {{
        var thread = document.querySelector('.thread');
        var temp = document.createElement('div');
        temp.innerHTML = html;
        while (temp.firstChild) {{
            thread.appendChild(temp.firstChild);
        }}
        // Reinitialize gallery handlers for new posts
        if (window.setupGalleryHandlers) {{
            window.setupGalleryHandlers();
        }}
    }}

    // Visibility-aware polling: pause when hidden, catch up when visible
    document.addEventListener('visibilitychange', function() {{
        if (document.hidden) {{
            if (timerId) {{
                clearTimeout(timerId);
                timerId = null;
            }}
        }} else {{
            if (stopped) return;
            var elapsed = Date.now() - lastPollTime;
            if (elapsed >= cfg.interval) {{
                poll();
            }} else {{
                schedule();
            }}
        }}
    }});

    // Initialize
    initRefreshButton();
    schedule();
}})();
"#,
        handle = html_escape::encode_text(&config.handle),
        post_id = html_escape::encode_text(&config.post_id),
        last_cid = html_escape::encode_text(&config.last_cid),
        initial_interval = config.initial_interval,
        max_interval = config.max_interval,
        disable_after = config.disable_after,
    )
}

pub fn base_template(title: &str, content: &str) -> String {
    base_template_with_options(title, content, TemplateOptions::default())
}

/// Render social meta tags (Open Graph and Twitter Cards) from SocialMeta options
fn render_social_meta(social: &SocialMeta) -> String {
    let mut tags = String::new();

    // Pre-compute escaped values to avoid repetition
    let escaped_title = social
        .title
        .map(|t| html_escape::encode_quoted_attribute(t).into_owned());
    let escaped_description = social.description.map(|d| {
        let truncated = truncate_for_description(d, 160);
        html_escape::encode_quoted_attribute(&truncated).into_owned()
    });
    let escaped_image = social
        .image_url
        .map(|u| html_escape::encode_quoted_attribute(u).into_owned());

    // Standard description meta tag
    if let Some(ref desc) = escaped_description {
        tags.push_str(&format!(
            r#"<meta name="description" content="{}">
    "#,
            desc
        ));
    }

    // Open Graph tags
    if let Some(og_type) = social.og_type {
        tags.push_str(&format!(
            r#"<meta property="og:type" content="{}">
    "#,
            html_escape::encode_quoted_attribute(og_type)
        ));
    }

    if let Some(ref title) = escaped_title {
        tags.push_str(&format!(
            r#"<meta property="og:title" content="{}">
    "#,
            title
        ));
    }

    if let Some(ref desc) = escaped_description {
        tags.push_str(&format!(
            r#"<meta property="og:description" content="{}">
    "#,
            desc
        ));
    }

    if let Some(url) = social.url {
        tags.push_str(&format!(
            r#"<meta property="og:url" content="{}">
    "#,
            html_escape::encode_quoted_attribute(url)
        ));
    }

    tags.push_str(
        r#"<meta property="og:site_name" content="sklonger">
    "#,
    );

    if let Some(ref image) = escaped_image {
        tags.push_str(&format!(
            r#"<meta property="og:image" content="{}">
    "#,
            image
        ));
    }

    // Twitter Card tags
    tags.push_str(
        r#"<meta name="twitter:card" content="summary">
    "#,
    );

    if let Some(ref title) = escaped_title {
        tags.push_str(&format!(
            r#"<meta name="twitter:title" content="{}">
    "#,
            title
        ));
    }

    if let Some(ref desc) = escaped_description {
        tags.push_str(&format!(
            r#"<meta name="twitter:description" content="{}">
    "#,
            desc
        ));
    }

    if let Some(ref image) = escaped_image {
        tags.push_str(&format!(
            r#"<meta name="twitter:image" content="{}">"#,
            image
        ));
    }

    tags
}

pub fn base_template_with_options(title: &str, content: &str, options: TemplateOptions) -> String {
    let favicon_tag = render_favicon_tag(options.favicon_url);
    let lang = options.lang.unwrap_or("en");
    let social_meta = options
        .social
        .as_ref()
        .map(render_social_meta)
        .unwrap_or_default();

    format!(
        r#"<!DOCTYPE html>
<html lang="{lang}">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>{title}</title>
    {social_meta}{favicon}{pwa_meta}
    <script>{theme_init}</script>
    <style>{css}</style>
</head>
<body>
{content}
<script>{theme_toggle}{sw_register}{gallery}</script>
</body>
</html>"#,
        lang = html_escape::encode_quoted_attribute(lang),
        title = html_escape::encode_text(title),
        social_meta = social_meta,
        favicon = favicon_tag,
        pwa_meta = PWA_META_TAGS,
        theme_init = THEME_SCRIPT,
        css = CSS_STYLES,
        content = content,
        theme_toggle = THEME_TOGGLE_SCRIPT,
        sw_register = SERVICE_WORKER_REGISTRATION,
        gallery = GALLERY_SCRIPT
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
    <div class="landing-tip">
        <div class="landing-tip-label">Tip</div>
        <p class="landing-tip-text">Or just replace <span class="highlight">bsky.app</span> with <span class="highlight">sklonger.app</span> in any Bluesky URL:</p>
        <p class="landing-tip-example"><a href="https://bsky.app/profile/nameshiv.bsky.social/post/3l3dpw5bpmi2j" target="_blank" rel="noopener">bsky.app/profile/...</a> <span class="highlight">→</span> <a href="https://sklonger.app/profile/nameshiv.bsky.social/post/3l3dpw5bpmi2j" target="_blank" rel="noopener">sklonger.app/profile/...</a></p>
    </div>
</main>
<div id="install-banner" class="install-banner">
    <span class="install-banner-text">Install Sklonger for quick access</span>
    <div class="install-banner-buttons">
        <button class="install-btn install-btn-primary" onclick="installApp()">Install</button>
        <button class="install-btn install-btn-dismiss" onclick="dismissInstall()">Not now</button>
    </div>
</div>
<footer class="landing-footer">
    <span class="support-link">Like this app? <a href="https://ko-fi.com/corybxyz" target="_blank" rel="noopener">Buy me a 🍵</a></span>
</footer>"#;

    let title = "Sklonger - Bluesky Thread Reader";
    let description = "Read Bluesky threads as single, clean pages. Paste any thread URL to view concatenated posts in a readable format. No login required.";

    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>{title}</title>
    <meta name="description" content="{description}">
    <meta property="og:type" content="website">
    <meta property="og:title" content="{title}">
    <meta property="og:description" content="{description}">
    <meta property="og:site_name" content="sklonger">
    <meta name="twitter:card" content="summary">
    <meta name="twitter:title" content="{title}">
    <meta name="twitter:description" content="{description}">
    {pwa_meta}
    <script>{theme_init}</script>
    <style>{css}</style>
</head>
<body>
{content}
<script>{theme_toggle}{sw_register}{pwa_install}</script>
</body>
</html>"#,
        title = html_escape::encode_text(title),
        description = html_escape::encode_text(description),
        pwa_meta = PWA_META_TAGS,
        theme_init = THEME_SCRIPT,
        css = CSS_STYLES,
        content = content,
        theme_toggle = THEME_TOGGLE_SCRIPT,
        sw_register = SERVICE_WORKER_REGISTRATION,
        pwa_install = PWA_INSTALL_SCRIPT
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

/// Options for streaming HTML head
pub struct StreamingHeadOptions<'a> {
    pub author_handle: &'a str,
    pub author_display_name: Option<&'a str>,
    pub avatar_url: Option<&'a str>,
    pub profile_url: &'a str,
    pub lang: Option<&'a str>,
    /// First post text for social sharing description (will be truncated)
    pub first_post_text: Option<&'a str>,
    /// Canonical URL for the thread (for og:url tag)
    pub thread_url: &'a str,
}

/// Truncate text to approximately the given length, breaking at word boundaries.
fn truncate_for_description(text: &str, max_len: usize) -> String {
    if text.len() <= max_len {
        return text.to_string();
    }

    let truncated = &text[..max_len];
    let break_point = truncated.rfind(' ').unwrap_or(max_len);
    format!("{}...", &text[..break_point])
}

/// Render the HTML head and header for streaming response.
/// This is sent immediately when we know the author info.
pub fn streaming_head(options: StreamingHeadOptions) -> String {
    let title = format!(
        "Thread by @{} - sklonger",
        html_escape::encode_text(options.author_handle)
    );

    let author_name = options.author_display_name.unwrap_or(options.author_handle);
    let avatar_html = render_avatar_html(options.avatar_url, author_name);
    let favicon_tag = render_favicon_tag(options.avatar_url);
    let lang = options.lang.unwrap_or("en");

    // Generate description from first post text for social sharing
    let og_title = format!("Thread by @{}", options.author_handle);
    let default_description = format!("A thread by {} on Bluesky", author_name);
    let description = options.first_post_text.unwrap_or(&default_description);

    // Reuse render_social_meta for consistency
    let social = SocialMeta {
        title: Some(&og_title),
        description: Some(description),
        url: Some(options.thread_url),
        image_url: options.avatar_url,
        og_type: Some("article"),
    };
    let social_meta = render_social_meta(&social);

    // Render header from template with interpolated values
    let header = HEADER_TEMPLATE
        .replace(
            "{profile_url}",
            html_escape::encode_quoted_attribute(options.profile_url).as_ref(),
        )
        .replace("{avatar}", &avatar_html)
        .replace(
            "{display_name}",
            html_escape::encode_text(author_name).as_ref(),
        )
        .replace(
            "{handle}",
            html_escape::encode_text(options.author_handle).as_ref(),
        );

    format!(
        r#"<!DOCTYPE html>
<html lang="{lang}">
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>{title}</title>
    {social_meta}{favicon}{pwa_meta}
    <script>{theme_init}</script>
    <style>{css}</style>
</head>
<body>
{header}
<main class="thread">
"#,
        lang = html_escape::encode_quoted_attribute(lang),
        title = html_escape::encode_text(&title),
        social_meta = social_meta,
        favicon = favicon_tag,
        pwa_meta = PWA_META_TAGS,
        theme_init = THEME_SCRIPT,
        css = CSS_STYLES,
        header = header,
    )
}

/// Render the closing HTML for a streaming response.
/// This includes the footer and closing tags.
/// If polling config is provided, the polling script is injected.
pub fn streaming_footer(original_post_url: &str, polling: Option<&PollingConfig>) -> String {
    let poll_script = polling.map(render_poll_script).unwrap_or_default();

    format!(
        r#"</main>
<footer>
    <a href="{url}" target="_blank" rel="noopener">View original on Bluesky</a>
    <span class="support-link">Like this app? <a href="https://ko-fi.com/corybxyz" target="_blank" rel="noopener">Buy me a 🍵</a></span>
</footer>
<script>
document.getElementById('loading-indicator')?.remove();
{theme_toggle}{sw_register}{gallery}{poll_script}
</script>
</body>
</html>"#,
        url = html_escape::encode_quoted_attribute(original_post_url),
        theme_toggle = THEME_TOGGLE_SCRIPT,
        sw_register = SERVICE_WORKER_REGISTRATION,
        gallery = GALLERY_SCRIPT,
        poll_script = poll_script
    )
}

/// Render the loading indicator to show at the bottom of the thread while more posts load.
pub fn streaming_loading_indicator() -> &'static str {
    r#"<div class="loading-indicator" id="loading-indicator">Loading more...</div>
"#
}

/// Render a post that should be inserted before the loading indicator.
/// Uses a small inline script to move the loading indicator to the end.
pub fn streaming_post_before_indicator(post_html: &str) -> String {
    // The script moves the loading indicator to be after the newly added post
    // by appending it to its parent (which moves it to the end)
    format!(
        r#"{post_html}<script>(function(){{var l=document.getElementById('loading-indicator');if(l)l.parentNode.appendChild(l);}})();</script>
"#,
        post_html = post_html
    )
}

/// Render an error that occurred mid-stream.
/// This closes the HTML properly so the page is still valid.
pub fn streaming_error(message: &str) -> String {
    format!(
        r#"<div class="stream-error">
    <p>Error loading thread: {}</p>
</div>
</main>
<footer>
    <a href="/">Try another thread</a>
</footer>
<script>
document.getElementById('loading-indicator')?.remove();
</script>
</body>
</html>"#,
        html_escape::encode_text(message)
    )
}
