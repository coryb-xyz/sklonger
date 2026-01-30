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
