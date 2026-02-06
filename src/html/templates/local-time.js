(function() {
    function formatLocalTime(date) {
        return date.toLocaleDateString(undefined, {
            month: 'short', day: 'numeric', year: 'numeric'
        }) + ' at ' + date.toLocaleTimeString(undefined, {
            hour: '2-digit', minute: '2-digit'
        });
    }

    function convertTimestamps() {
        var times = document.querySelectorAll('time[datetime]');
        for (var i = 0; i < times.length; i++) {
            var el = times[i];
            if (el.getAttribute('data-localized')) continue;
            var dt = el.getAttribute('datetime');
            var date = new Date(dt);
            if (!isNaN(date.getTime())) {
                el.textContent = formatLocalTime(date);
                el.setAttribute('data-localized', '1');
            }
        }
    }

    window.formatLocalTime = formatLocalTime;
    window.convertTimestamps = convertTimestamps;
    convertTimestamps();
})();
