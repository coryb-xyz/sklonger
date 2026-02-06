(function() {
    var pct;
    // Migrate from old named sizes to percentage
    var oldSize = localStorage.getItem('fontSize');
    if (oldSize) {
        var map = { small: 100, medium: 110, large: 130 };
        pct = map[oldSize] || 100;
        localStorage.setItem('fontSizePct', String(pct));
        localStorage.removeItem('fontSize');
    } else {
        pct = parseInt(localStorage.getItem('fontSizePct'), 10) || 100;
    }
    if (pct !== 100) {
        document.documentElement.style.setProperty('--content-font-size', (16 * pct / 100) + 'px');
        document.documentElement.style.setProperty('--content-font-size-sm', (14 * pct / 100) + 'px');
    }
})();
