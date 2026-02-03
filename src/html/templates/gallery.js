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
    let touchStartX = 0;
    let touchStartY = 0;
    let touchCurrentX = 0;
    let isDragging = false;
    let isTransitioning = false;
    let closedViaPopstate = false;
    const SWIPE_THRESHOLD = 50;

    // Zoom state
    let zoomLevel = 1;
    let zoomPanX = 0;
    let zoomPanY = 0;
    let initialPinchDistance = 0;
    let isPinching = false;
    let lastTapTime = 0;
    const DOUBLE_TAP_DELAY = 300;
    const MIN_ZOOM = 1;
    const MAX_ZOOM = 4;

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
                <div class="gallery-media">
                    <div class="gallery-track"></div>
                </div>
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
        overlay.addEventListener('touchstart', handleTouchStart, { passive: false });
        overlay.addEventListener('touchmove', handleTouchMove, { passive: false });
        overlay.addEventListener('touchend', handleTouchEnd, { passive: true });

        document.addEventListener('keydown', handleKeyboard);

        // Handle browser back button
        window.addEventListener('popstate', handlePopstate);
    }

    function handlePopstate(e) {
        if (overlay && overlay.classList.contains('active')) {
            closedViaPopstate = true;
            closeGallery();
        }
    }

    function handleTouchStart(e) {
        if (isTransitioning) return;

        // Handle double-tap to zoom
        const now = Date.now();
        if (e.touches.length === 1 && now - lastTapTime < DOUBLE_TAP_DELAY) {
            e.preventDefault();
            handleDoubleTap(e.touches[0]);
            lastTapTime = 0;
            return;
        }
        lastTapTime = now;

        // Handle pinch zoom
        if (e.touches.length === 2) {
            isPinching = true;
            initialPinchDistance = getPinchDistance(e.touches);
            return;
        }

        // Handle single touch (pan or swipe)
        const touch = e.touches[0];
        touchStartX = touch.clientX;
        touchStartY = touch.clientY;
        touchCurrentX = touch.clientX;
        isDragging = false;
    }

    function getPinchDistance(touches) {
        const dx = touches[0].clientX - touches[1].clientX;
        const dy = touches[0].clientY - touches[1].clientY;
        return Math.sqrt(dx * dx + dy * dy);
    }

    function handleDoubleTap(touch) {
        const currentImage = getCurrentImage();
        if (!currentImage) return;

        if (zoomLevel > MIN_ZOOM) {
            // Zoom out to original
            resetZoom();
        } else {
            // Zoom in to 2x at tap location
            const rect = currentImage.getBoundingClientRect();
            const x = touch.clientX - rect.left;
            const y = touch.clientY - rect.top;

            zoomLevel = 2;
            // Center zoom on tap point
            zoomPanX = (rect.width / 2 - x) * (zoomLevel - 1);
            zoomPanY = (rect.height / 2 - y) * (zoomLevel - 1);

            applyZoomTransform(currentImage, true);
        }
    }

    function handleTouchMove(e) {
        if (isTransitioning) return;

        // Handle pinch zoom
        if (e.touches.length === 2 && isPinching) {
            e.preventDefault();
            const currentDistance = getPinchDistance(e.touches);
            const scale = currentDistance / initialPinchDistance;
            const newZoom = Math.max(MIN_ZOOM, Math.min(MAX_ZOOM, zoomLevel * scale));

            const currentImage = getCurrentImage();
            if (currentImage) {
                applyZoomTransform(currentImage, false, newZoom);
            }
            return;
        }

        // If zoomed in, handle panning instead of swiping
        if (zoomLevel > MIN_ZOOM && e.touches.length === 1) {
            e.preventDefault();
            const touch = e.touches[0];
            const deltaX = touch.clientX - touchStartX;
            const deltaY = touch.clientY - touchStartY;

            zoomPanX += deltaX;
            zoomPanY += deltaY;

            touchStartX = touch.clientX;
            touchStartY = touch.clientY;

            const currentImage = getCurrentImage();
            if (currentImage) {
                applyZoomTransform(currentImage, false);
            }
            return;
        }

        // Handle carousel swipe (only when not zoomed)
        const touch = e.touches[0];
        const deltaX = touch.clientX - touchStartX;
        const deltaY = Math.abs(touch.clientY - touchStartY);

        // Only start dragging if horizontal movement is dominant
        if (!isDragging && Math.abs(deltaX) > 10 && Math.abs(deltaX) > deltaY) {
            isDragging = true;
        }

        if (isDragging) {
            e.preventDefault(); // Prevent scrolling while dragging
            touchCurrentX = touch.clientX;

            // Apply drag with resistance at boundaries
            let dragDistance = deltaX;
            const atStart = currentMediaIndex === 0;
            const atEnd = currentMediaIndex === mediaItems.length - 1;

            if ((atStart && dragDistance > 0) || (atEnd && dragDistance < 0)) {
                dragDistance = dragDistance * 0.3; // Elastic resistance
            }

            const track = container.querySelector('.gallery-track');
            if (track) {
                track.style.transition = 'none';
                // Base position based on current index
                const baseOffset = -currentMediaIndex * window.innerWidth;
                track.style.transform = `translateX(${baseOffset + dragDistance}px)`;
            }
        }
    }

    function handleTouchEnd(e) {
        if (isTransitioning) return;

        // Handle pinch zoom end
        if (isPinching) {
            isPinching = false;
            const currentImage = getCurrentImage();
            if (currentImage) {
                const currentDistance = e.changedTouches.length === 2 ? getPinchDistance(e.changedTouches) : initialPinchDistance;
                const scale = currentDistance / initialPinchDistance;
                zoomLevel = Math.max(MIN_ZOOM, Math.min(MAX_ZOOM, zoomLevel * scale));
                applyZoomTransform(currentImage, true);
            }
            return;
        }

        // Don't handle carousel swipe if zoomed in
        if (zoomLevel > MIN_ZOOM) {
            return;
        }

        if (!isDragging) {
            return;
        }

        const swipeDistance = touchCurrentX - touchStartX;
        const velocity = Math.abs(swipeDistance);

        // Determine if we should change slides
        const shouldChange = velocity > SWIPE_THRESHOLD || Math.abs(swipeDistance) > window.innerWidth * 0.25;

        if (shouldChange) {
            if (swipeDistance > 0 && currentMediaIndex > 0) {
                showPrevious();
            } else if (swipeDistance < 0 && currentMediaIndex < mediaItems.length - 1) {
                showNext();
            } else {
                // Snap back to current
                updateSlidePosition(false);
            }
        } else {
            // Snap back to current
            updateSlidePosition(false);
        }

        isDragging = false;
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

        // Push history state so back button closes gallery
        history.pushState({ galleryOpen: true }, '');

        showMedia(currentMediaIndex);
    }

    function closeGallery() {
        if (!overlay) return;

        overlay.classList.remove('active');
        document.body.style.overflow = '';

        // Clean up slides
        const track = container.querySelector('.gallery-track');
        if (track) {
            track.innerHTML = '';
        } else {
            container.innerHTML = '';
        }

        // Go back in history if we didn't get here via popstate
        if (!closedViaPopstate) {
            history.back();
        }
        closedViaPopstate = false;
    }

    function showMedia(index, animate = false) {
        if (index < 0 || index >= mediaItems.length) return;

        // Reset zoom when changing slides
        resetZoom();

        currentMediaIndex = index;
        renderSlides();
        updateSlidePosition(animate);

        counter.textContent = (currentMediaIndex + 1) + ' / ' + mediaItems.length;

        updateNavButton(prevBtn, currentMediaIndex > 0);
        updateNavButton(nextBtn, currentMediaIndex < mediaItems.length - 1);
    }

    function renderSlides() {
        const track = container.querySelector('.gallery-track');
        if (!track) return;

        // Clear existing slides
        track.innerHTML = '';

        // Render ALL media items as slides
        mediaItems.forEach(function(item, idx) {
            track.appendChild(createSlide(idx));
        });
    }

    function createSlide(idx) {
        const item = mediaItems[idx];
        const slide = document.createElement('div');
        slide.className = 'gallery-slide';

        if (item.type === 'image') {
            const img = document.createElement('img');
            img.className = 'gallery-image';
            img.src = item.src;
            img.alt = item.alt;
            slide.appendChild(img);
        } else if (item.type === 'video') {
            const video = document.createElement('video');
            video.className = 'gallery-video';
            video.controls = true;
            video.setAttribute('aria-label', item.alt);

            // Only autoplay the current video
            if (idx === currentMediaIndex) {
                video.autoplay = true;
            }

            const source = document.createElement('source');
            source.src = item.src;
            source.type = 'application/x-mpegURL';
            video.appendChild(source);
            slide.appendChild(video);
        }

        return slide;
    }

    function updateSlidePosition(animate) {
        const track = container.querySelector('.gallery-track');
        if (!track) return;

        if (animate) {
            isTransitioning = true;
            track.style.transition = 'transform 0.3s ease-out';

            // Remove transition flag after animation completes
            setTimeout(function() {
                isTransitioning = false;
            }, 300);
        } else {
            track.style.transition = '';
        }

        // Each slide is 100vw wide, translate to show the current slide
        // For index 0, translate 0; for index 1, translate -100vw, etc.
        const translateAmount = -currentMediaIndex * 100;
        track.style.transform = `translateX(${translateAmount}vw)`;
    }

    function updateNavButton(btn, enabled) {
        btn.style.opacity = enabled ? '1' : '0.3';
        btn.style.pointerEvents = enabled ? 'auto' : 'none';
    }

    function showPrevious() {
        if (currentMediaIndex > 0 && !isTransitioning) {
            showMedia(currentMediaIndex - 1, true);
        }
    }

    function showNext() {
        if (currentMediaIndex < mediaItems.length - 1 && !isTransitioning) {
            showMedia(currentMediaIndex + 1, true);
        }
    }

    function getCurrentImage() {
        const slides = container.querySelectorAll('.gallery-slide');
        if (slides[currentMediaIndex]) {
            return slides[currentMediaIndex].querySelector('.gallery-image');
        }
        return null;
    }

    function applyZoomTransform(img, animate, tempZoom) {
        const zoom = tempZoom !== undefined ? tempZoom : zoomLevel;

        if (animate) {
            img.style.transition = 'transform 0.3s ease-out';
        } else {
            img.style.transition = 'none';
        }

        if (zoom > MIN_ZOOM) {
            img.classList.add('zoomed');
            img.style.transform = `scale(${zoom}) translate(${zoomPanX / zoom}px, ${zoomPanY / zoom}px)`;
        } else {
            img.classList.remove('zoomed');
            img.style.transform = '';
        }
    }

    function resetZoom() {
        zoomLevel = MIN_ZOOM;
        zoomPanX = 0;
        zoomPanY = 0;

        const currentImage = getCurrentImage();
        if (currentImage) {
            applyZoomTransform(currentImage, true);
        }
    }

    // Setup click handlers for images and videos
    function setupMediaHandlers() {
        const posts = document.querySelectorAll('.post');

        posts.forEach(function(post, postIndex) {
            // Handle image clicks
            const imageLinks = post.querySelectorAll('.embed-image-link');
            imageLinks.forEach(function(link, mediaIndex) {
                // Remove any existing handlers to prevent duplicates
                link.removeEventListener('click', link._galleryClickHandler);

                // Create and store the handler
                link._galleryClickHandler = function(e) {
                    e.preventDefault();
                    e.stopPropagation();
                    openGallery(postIndex, mediaIndex);
                };

                link.addEventListener('click', link._galleryClickHandler);
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
