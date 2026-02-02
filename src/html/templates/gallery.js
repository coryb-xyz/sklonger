// Gallery viewer for images and videos
(function() {
    let currentMediaIndex = 0;
    let mediaItems = [];
    let overlay = null;
    let container = null;
    let prevBtn = null;
    let nextBtn = null;
    let closeBtn = null;
    let counter = null;
    let mediaElement = null;
    let touchStartX = 0;
    const SWIPE_THRESHOLD = 50;

    function createOverlay() {
        overlay = document.createElement('div');
        overlay.className = 'gallery-overlay';
        overlay.innerHTML = `
            <div class="gallery-container">
                <button class="gallery-close" aria-label="Close gallery">
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <line x1="18" y1="6" x2="6" y2="18"></line>
                        <line x1="6" y1="6" x2="18" y2="18"></line>
                    </svg>
                </button>
                <button class="gallery-nav gallery-prev" aria-label="Previous image">
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <polyline points="15 18 9 12 15 6"></polyline>
                    </svg>
                </button>
                <div class="gallery-media"></div>
                <button class="gallery-nav gallery-next" aria-label="Next image">
                    <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <polyline points="9 18 15 12 9 6"></polyline>
                    </svg>
                </button>
                <div class="gallery-counter"></div>
            </div>
        `;
        document.body.appendChild(overlay);

        container = overlay.querySelector('.gallery-media');
        prevBtn = overlay.querySelector('.gallery-prev');
        nextBtn = overlay.querySelector('.gallery-next');
        closeBtn = overlay.querySelector('.gallery-close');
        counter = overlay.querySelector('.gallery-counter');

        // Event listeners
        closeBtn.addEventListener('click', closeGallery);
        prevBtn.addEventListener('click', showPrevious);
        nextBtn.addEventListener('click', showNext);
        overlay.addEventListener('click', function(e) {
            if (e.target === overlay || e.target === container) {
                closeGallery();
            }
        });

        // Touch swipe support
        overlay.addEventListener('touchstart', handleTouchStart, { passive: true });
        overlay.addEventListener('touchend', handleTouchEnd, { passive: true });

        document.addEventListener('keydown', handleKeyboard);
    }

    function handleTouchStart(e) {
        touchStartX = e.changedTouches[0].screenX;
    }

    function handleTouchEnd(e) {
        const swipeDistance = e.changedTouches[0].screenX - touchStartX;
        if (Math.abs(swipeDistance) < SWIPE_THRESHOLD) return;

        if (swipeDistance > 0) {
            showPrevious();
        } else {
            showNext();
        }
    }

    function handleKeyboard(e) {
        if (!overlay || !overlay.classList.contains('active')) return;

        switch(e.key) {
            case 'Escape':
                closeGallery();
                break;
            case 'ArrowLeft':
                showPrevious();
                break;
            case 'ArrowRight':
                showNext();
                break;
        }
    }

    function openGallery(postIndex, mediaIndex) {
        if (!overlay) createOverlay();

        currentMediaIndex = mediaIndex;

        // Collect all media items from this post
        const post = document.querySelectorAll('.post')[postIndex];
        if (!post) return;

        mediaItems = [];

        // Collect images
        const images = post.querySelectorAll('.embed-image-link');
        images.forEach(function(link) {
            const img = link.querySelector('img');
            mediaItems.push({
                type: 'image',
                src: link.href,
                alt: img ? img.alt : ''
            });
        });

        // Collect videos
        const videos = post.querySelectorAll('.embed-video video');
        videos.forEach(function(video) {
            const source = video.querySelector('source');
            mediaItems.push({
                type: 'video',
                src: source ? source.src : '',
                alt: video.getAttribute('aria-label') || 'Video'
            });
        });

        if (mediaItems.length === 0) return;

        overlay.classList.add('active');
        document.body.style.overflow = 'hidden';

        showMedia(currentMediaIndex);
    }

    function closeGallery() {
        if (!overlay) return;

        overlay.classList.remove('active');
        document.body.style.overflow = '';

        if (mediaElement) {
            if (mediaElement.tagName === 'VIDEO') {
                mediaElement.pause();
            }
            container.innerHTML = '';
            mediaElement = null;
        }
    }

    function showMedia(index) {
        if (index < 0 || index >= mediaItems.length) return;

        currentMediaIndex = index;
        const item = mediaItems[index];

        // Clear previous media
        container.innerHTML = '';

        if (item.type === 'image') {
            mediaElement = document.createElement('img');
            mediaElement.className = 'gallery-image';
            mediaElement.src = item.src;
            mediaElement.alt = item.alt;
        } else if (item.type === 'video') {
            mediaElement = document.createElement('video');
            mediaElement.className = 'gallery-video';
            mediaElement.controls = true;
            mediaElement.autoplay = true;
            mediaElement.setAttribute('aria-label', item.alt);

            const source = document.createElement('source');
            source.src = item.src;
            source.type = 'application/x-mpegURL';
            mediaElement.appendChild(source);
        }

        container.appendChild(mediaElement);

        counter.textContent = (currentMediaIndex + 1) + ' / ' + mediaItems.length;

        updateNavButton(prevBtn, currentMediaIndex > 0);
        updateNavButton(nextBtn, currentMediaIndex < mediaItems.length - 1);
    }

    function updateNavButton(btn, enabled) {
        btn.style.opacity = enabled ? '1' : '0.3';
        btn.style.pointerEvents = enabled ? 'auto' : 'none';
    }

    function showPrevious() {
        if (currentMediaIndex > 0) {
            showMedia(currentMediaIndex - 1);
        }
    }

    function showNext() {
        if (currentMediaIndex < mediaItems.length - 1) {
            showMedia(currentMediaIndex + 1);
        }
    }

    // Setup click handlers for images and videos
    function setupMediaHandlers() {
        const posts = document.querySelectorAll('.post');

        posts.forEach(function(post, postIndex) {
            // Handle image clicks
            const imageLinks = post.querySelectorAll('.embed-image-link');
            imageLinks.forEach(function(link, mediaIndex) {
                link.addEventListener('click', function(e) {
                    e.preventDefault();
                    openGallery(postIndex, mediaIndex);
                });
            });

            // Handle video container clicks (not the video controls)
            const videoContainers = post.querySelectorAll('.embed-video');
            videoContainers.forEach(function(container, index) {
                container.addEventListener('click', function(e) {
                    // Only open gallery if clicking on container, not video controls
                    if (e.target === container) {
                        const imageCount = post.querySelectorAll('.embed-image-link').length;
                        openGallery(postIndex, imageCount + index);
                    }
                });
            });
        });
    }

    // Initialize when DOM is ready
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', setupMediaHandlers);
    } else {
        setupMediaHandlers();
    }

    // Expose function for dynamically added posts (polling/streaming)
    window.setupGalleryHandlers = setupMediaHandlers;
})();
