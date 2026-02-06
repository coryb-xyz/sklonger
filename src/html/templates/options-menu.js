(function() {
    var optionsBtn = document.getElementById('options-btn');
    var overlay = document.getElementById('options-overlay');
    var menu = document.getElementById('options-menu');
    if (!optionsBtn || !overlay || !menu) return;

    var isOpen = false;
    var menuContent = menu.querySelector('.options-menu-content');

    function openMenu() {
        if (isOpen) return;
        isOpen = true;
        overlay.style.display = 'block';
        // Force reflow so transition triggers
        overlay.offsetHeight;
        overlay.classList.add('active');
        overlay.setAttribute('aria-hidden', 'false');
        menu.classList.add('active');
        menu.setAttribute('aria-hidden', 'false');
        document.body.style.overflow = 'hidden';
    }

    function closeMenu() {
        if (!isOpen) return;
        isOpen = false;
        menu.style.transform = '';
        overlay.style.backgroundColor = '';
        overlay.classList.remove('active');
        overlay.setAttribute('aria-hidden', 'true');
        menu.classList.remove('active');
        menu.setAttribute('aria-hidden', 'true');
        document.body.style.overflow = '';
        // Hide overlay after transition
        setTimeout(function() {
            if (!isOpen) overlay.style.display = 'none';
        }, 300);
    }

    optionsBtn.addEventListener('click', function() {
        if (isOpen) closeMenu();
        else openMenu();
    });

    overlay.addEventListener('click', closeMenu);

    document.addEventListener('keydown', function(e) {
        if (e.key === 'Escape' && isOpen) {
            closeMenu();
        }
    });

    // Swipe-to-dismiss
    var startY = 0;
    var currentY = 0;
    var dragging = false;
    var menuHeight = 0;
    var DISMISS_THRESHOLD = 0.3;

    menu.addEventListener('touchstart', function(e) {
        if (!isOpen) return;
        // Only allow swipe when content is scrolled to top
        if (menuContent && menuContent.scrollTop > 0) return;
        startY = e.touches[0].clientY;
        currentY = startY;
        menuHeight = menu.offsetHeight;
        dragging = false;
    }, { passive: true });

    menu.addEventListener('touchmove', function(e) {
        if (!isOpen || !menuHeight) return;
        currentY = e.touches[0].clientY;
        var dy = Math.max(0, currentY - startY);

        // Start dragging after a small threshold to avoid accidental triggers
        if (!dragging && dy > 5) {
            dragging = true;
            menu.style.transition = 'none';
        }

        if (dragging) {
            e.preventDefault();
            menu.style.transform = 'translateY(' + dy + 'px)';
            // Fade overlay proportionally
            var progress = Math.min(dy / menuHeight, 1);
            overlay.style.backgroundColor = 'rgba(0, 0, 0, ' + (0.5 * (1 - progress)) + ')';
        }
    }, { passive: false });

    menu.addEventListener('touchend', function() {
        if (!dragging) return;
        dragging = false;
        menu.style.transition = '';

        var dy = currentY - startY;
        if (dy > menuHeight * DISMISS_THRESHOLD) {
            closeMenu();
        } else {
            // Snap back
            menu.style.transform = '';
            overlay.style.backgroundColor = '';
        }
    }, { passive: true });

    // Hide auto-refresh row and notice on pages without a refresh button (landing, error)
    var autoRefreshRow = document.getElementById('auto-refresh-row');
    var lastPostNotice = document.getElementById('last-post-notice');
    var refreshBtn = document.getElementById('refresh-btn');
    if (!refreshBtn) {
        if (autoRefreshRow) autoRefreshRow.style.display = 'none';
        if (lastPostNotice) lastPostNotice.style.display = 'none';
    }

    // Auto-refresh toggle
    var autoRefreshToggle = document.getElementById('auto-refresh-toggle');
    if (autoRefreshToggle && refreshBtn) {
        // Initial state driven by thread staleness (not persisted across sessions)
        var isEnabled = !window._threadStale;
        autoRefreshToggle.setAttribute('aria-checked', isEnabled ? 'true' : 'false');

        autoRefreshToggle.addEventListener('click', function() {
            isEnabled = !isEnabled;
            autoRefreshToggle.setAttribute('aria-checked', isEnabled ? 'true' : 'false');
            if (window.setAutoRefreshEnabled) {
                window.setAutoRefreshEnabled(isEnabled);
            }
        });

        // Listen for backend-detected staleness
        document.addEventListener('threadstale', function() {
            if (isEnabled) {
                isEnabled = false;
                autoRefreshToggle.setAttribute('aria-checked', 'false');
            }
            updateLastPostNotice();
        });
    }

    // Last post time notice
    function formatRelativeTime(date) {
        var now = new Date();
        var diffMs = now - date;
        var diffMin = Math.floor(diffMs / 60000);
        var diffHr = Math.floor(diffMs / 3600000);
        var diffDay = Math.floor(diffMs / 86400000);
        if (diffMin < 1) return 'just now';
        if (diffMin < 60) return diffMin + (diffMin === 1 ? ' minute ago' : ' minutes ago');
        if (diffHr < 24) return diffHr + (diffHr === 1 ? ' hour ago' : ' hours ago');
        return diffDay + (diffDay === 1 ? ' day ago' : ' days ago');
    }

    function updateLastPostNotice() {
        if (!lastPostNotice || !refreshBtn || !window._lastPostTime) return;
        var lastDate = new Date(window._lastPostTime);
        if (isNaN(lastDate.getTime())) return;
        var formatted = window.formatLocalTime
            ? window.formatLocalTime(lastDate)
            : lastDate.toLocaleString();
        lastPostNotice.textContent = 'Last post: ' + formatted + ' (' + formatRelativeTime(lastDate) + ')';
    }

    updateLastPostNotice();
})();
