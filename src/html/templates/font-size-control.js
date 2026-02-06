(function() {
    var decreaseBtn = document.getElementById('font-size-decrease');
    var increaseBtn = document.getElementById('font-size-increase');
    var display = document.getElementById('font-size-display');
    if (!decreaseBtn || !increaseBtn || !display) return;

    var MIN = 80;
    var MAX = 140;
    var STEP = 10;
    var DEFAULT = 100;
    var BASE_FONT = 16;
    var BASE_FONT_SM = 14;

    function getCurrentPct() {
        var stored = parseInt(localStorage.getItem('fontSizePct'), 10);
        if (isNaN(stored) || stored < MIN || stored > MAX) return DEFAULT;
        return stored;
    }

    function applySize(pct) {
        if (pct === DEFAULT) {
            document.documentElement.style.removeProperty('--content-font-size');
            document.documentElement.style.removeProperty('--content-font-size-sm');
        } else {
            document.documentElement.style.setProperty('--content-font-size', (BASE_FONT * pct / 100) + 'px');
            document.documentElement.style.setProperty('--content-font-size-sm', (BASE_FONT_SM * pct / 100) + 'px');
        }
        localStorage.setItem('fontSizePct', String(pct));
        updateUI(pct);
    }

    function updateUI(pct) {
        display.textContent = pct + '%';
        decreaseBtn.disabled = (pct <= MIN);
        increaseBtn.disabled = (pct >= MAX);
    }

    // Initialize display
    updateUI(getCurrentPct());

    decreaseBtn.addEventListener('click', function() {
        var pct = getCurrentPct();
        if (pct > MIN) applySize(pct - STEP);
    });

    increaseBtn.addEventListener('click', function() {
        var pct = getCurrentPct();
        if (pct < MAX) applySize(pct + STEP);
    });
})();
