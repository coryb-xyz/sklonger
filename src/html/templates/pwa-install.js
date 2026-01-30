var deferredPrompt = null;
var installBanner = null;

window.addEventListener('beforeinstallprompt', function(e) {
    e.preventDefault();
    deferredPrompt = e;
    showInstallBanner();
});

function showInstallBanner() {
    installBanner = document.getElementById('install-banner');
    if (installBanner) {
        installBanner.style.display = 'flex';
    }
}

function installApp() {
    if (!deferredPrompt) return;
    deferredPrompt.prompt();
    deferredPrompt.userChoice.then(function(result) {
        deferredPrompt = null;
        if (installBanner) {
            installBanner.style.display = 'none';
        }
    });
}

function dismissInstall() {
    if (installBanner) {
        installBanner.style.display = 'none';
    }
    deferredPrompt = null;
}

window.addEventListener('appinstalled', function() {
    if (installBanner) {
        installBanner.style.display = 'none';
    }
    deferredPrompt = null;
});
