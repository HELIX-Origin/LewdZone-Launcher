<svelte:options runes={true} />

<script lang="ts">
  interface Props {
    title: string;
    screenshots: string[];
  }

  let { title, screenshots }: Props = $props();

  let activeIndex = $state(0);
  let isLightboxOpen = $state(false);

  // Keep active index in bounds if screenshots array changes
  $effect(() => {
    if (screenshots.length > 0 && activeIndex >= screenshots.length) {
      activeIndex = 0;
    }
  });

  function prevImage() {
    if (screenshots.length <= 1) return;
    activeIndex = (activeIndex - 1 + screenshots.length) % screenshots.length;
  }

  function nextImage() {
    if (screenshots.length <= 1) return;
    activeIndex = (activeIndex + 1) % screenshots.length;
  }

  function selectImage(idx: number) {
    if (idx >= 0 && idx < screenshots.length) {
      activeIndex = idx;
    }
  }

  function toggleLightbox() {
    isLightboxOpen = !isLightboxOpen;
  }

  function handleKeydown(e: KeyboardEvent) {
    if (isLightboxOpen) {
      if (e.key === "Escape") {
        isLightboxOpen = false;
      } else if (e.key === "ArrowLeft") {
        prevImage();
      } else if (e.key === "ArrowRight") {
        nextImage();
      }
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if screenshots && screenshots.length > 0}
  <section class="section-card" aria-label="Game Preview Images">
    <div class="carousel-head">
      <h2>Artwork & Media</h2>
      <span class="counter">{activeIndex + 1} / {screenshots.length}</span>
    </div>

    <div class="carousel-stage">
      {#if screenshots.length > 1}
        <button
          type="button"
          class="nav-arrow left"
          onclick={prevImage}
          aria-label="Previous image"
        >
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="15 18 9 12 15 6"></polyline>
          </svg>
        </button>
      {/if}

      <button
        type="button"
        class="stage-img-btn"
        onclick={toggleLightbox}
        aria-label="Click to enlarge image"
      >
        <img
          class="stage-img"
          src={screenshots[activeIndex]}
          alt={`${title} preview screenshot ${activeIndex + 1}`}
        />
        <div class="stage-overlay">
          <span class="zoom-badge">🔍 Click for Fullscreen</span>
        </div>
      </button>

      {#if screenshots.length > 1}
        <button
          type="button"
          class="nav-arrow right"
          onclick={nextImage}
          aria-label="Next image"
        >
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round">
            <polyline points="9 18 15 12 9 6"></polyline>
          </svg>
        </button>
      {/if}
    </div>

    <!-- Thumbnail Strip (up to 10 images) -->
    {#if screenshots.length > 1}
      <div class="thumbnail-strip" role="tablist" aria-label="Thumbnails">
        {#each screenshots as thumbUrl, idx (thumbUrl)}
          <button
            type="button"
            class="thumb-btn"
            class:active={idx === activeIndex}
            onclick={() => selectImage(idx)}
            aria-label={`View image ${idx + 1}`}
            role="tab"
            aria-selected={idx === activeIndex}
          >
            <img src={thumbUrl} alt="" loading="lazy" />
          </button>
        {/each}
      </div>
    {/if}
  </section>
{/if}

<!-- Lightbox Modal -->
{#if isLightboxOpen && screenshots.length > 0}
  <div class="lightbox" role="dialog" aria-modal="true">
    <button class="lightbox-close" onclick={toggleLightbox} aria-label="Close fullscreen view">✕</button>
    {#if screenshots.length > 1}
      <button class="lightbox-nav left" onclick={prevImage} aria-label="Previous">❮</button>
    {/if}
    <img
      class="lightbox-img"
      src={screenshots[activeIndex]}
      alt={`${title} full view`}
    />
    {#if screenshots.length > 1}
      <button class="lightbox-nav right" onclick={nextImage} aria-label="Next">❯</button>
    {/if}
    <div class="lightbox-caption">{activeIndex + 1} of {screenshots.length}</div>
  </div>
{/if}

<style>
  .section-card {
    background: var(--lz-surface, #1e1e24);
    border: 1px solid var(--lz-border, rgba(255, 255, 255, 0.08));
    border-radius: 8px;
    padding: 20px;
    margin-bottom: 24px;
  }

  .carousel-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 14px;
  }

  .carousel-head h2 {
    font-size: 16px;
    font-weight: 600;
    margin: 0;
    color: var(--lz-text, #fff);
  }

  .counter {
    font-size: 12px;
    color: var(--lz-text-muted, #8e8e93);
    font-variant-numeric: tabular-nums;
    font-weight: 500;
  }

  .carousel-stage {
    position: relative;
    width: 100%;
    aspect-ratio: 16 / 9;
    max-height: 480px;
    background: #000;
    border-radius: 6px;
    overflow: hidden;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .stage-img-btn {
    all: unset;
    width: 100%;
    height: 100%;
    cursor: zoom-in;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .stage-img {
    max-width: 100%;
    max-height: 100%;
    object-fit: contain;
  }

  .stage-overlay {
    position: absolute;
    bottom: 10px;
    right: 10px;
    background: rgba(0, 0, 0, 0.65);
    padding: 4px 8px;
    border-radius: 4px;
    font-size: 11px;
    color: #fff;
    opacity: 0;
    transition: opacity 0.2s ease;
    pointer-events: none;
  }

  .carousel-stage:hover .stage-overlay {
    opacity: 1;
  }

  .nav-arrow {
    position: absolute;
    top: 50%;
    transform: translateY(-50%);
    width: 38px;
    height: 38px;
    background: rgba(0, 0, 0, 0.6);
    border: 1px solid rgba(255, 255, 255, 0.15);
    border-radius: 50%;
    color: #fff;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    z-index: 5;
    transition: background 0.15s ease, transform 0.15s ease;
  }

  .nav-arrow svg {
    width: 18px;
    height: 18px;
  }

  .nav-arrow:hover {
    background: rgba(0, 0, 0, 0.9);
    transform: translateY(-50%) scale(1.08);
  }

  .nav-arrow.left {
    left: 12px;
  }

  .nav-arrow.right {
    right: 12px;
  }

  .thumbnail-strip {
    display: flex;
    gap: 8px;
    margin-top: 10px;
    overflow-x: auto;
    padding-bottom: 4px;
  }

  .thumbnail-strip::-webkit-scrollbar {
    height: 4px;
  }

  .thumbnail-strip::-webkit-scrollbar-thumb {
    background: var(--lz-surface-2, #2c2c34);
    border-radius: 2px;
  }

  .thumb-btn {
    all: unset;
    flex: 0 0 84px;
    aspect-ratio: 16 / 9;
    border-radius: 4px;
    overflow: hidden;
    cursor: pointer;
    border: 2px solid transparent;
    opacity: 0.65;
    transition: border-color 0.15s ease, opacity 0.15s ease;
  }

  .thumb-btn:hover {
    opacity: 0.9;
  }

  .thumb-btn.active {
    border-color: var(--lz-cyan, #00dfd8);
    opacity: 1;
  }

  .thumb-btn img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  /* Lightbox */
  .lightbox {
    position: fixed;
    inset: 0;
    z-index: 999;
    background: rgba(0, 0, 0, 0.92);
    display: flex;
    align-items: center;
    justify-content: center;
    backdrop-filter: blur(8px);
  }

  .lightbox-img {
    max-width: 90vw;
    max-height: 85vh;
    object-fit: contain;
    border-radius: 4px;
    box-shadow: 0 10px 40px rgba(0, 0, 0, 0.8);
  }

  .lightbox-close {
    position: absolute;
    top: 20px;
    right: 24px;
    background: rgba(255, 255, 255, 0.1);
    border: none;
    border-radius: 50%;
    width: 36px;
    height: 36px;
    color: #fff;
    font-size: 20px;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: background 0.15s ease;
  }

  .lightbox-close:hover {
    background: rgba(255, 255, 255, 0.25);
  }

  .lightbox-nav {
    position: absolute;
    top: 50%;
    transform: translateY(-50%);
    background: rgba(255, 255, 255, 0.12);
    border: none;
    color: #fff;
    font-size: 26px;
    padding: 16px 20px;
    border-radius: 4px;
    cursor: pointer;
    transition: background 0.15s ease;
  }

  .lightbox-nav:hover {
    background: rgba(255, 255, 255, 0.28);
  }

  .lightbox-nav.left {
    left: 24px;
  }

  .lightbox-nav.right {
    right: 24px;
  }

  .lightbox-caption {
    position: absolute;
    bottom: 20px;
    left: 50%;
    transform: translateX(-50%);
    color: rgba(255, 255, 255, 0.8);
    font-size: 13px;
    background: rgba(0, 0, 0, 0.6);
    padding: 4px 12px;
    border-radius: 12px;
  }
</style>
