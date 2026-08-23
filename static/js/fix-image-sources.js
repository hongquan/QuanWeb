// The Bunny CDN sometimes is blocked in Viet Nam.
// If the `<img>` elements with `src` URL from "https://quan-images.b-cdn.net" fail to load,
// the script will replace the hostname of them with "img.quan.hoabinh.vn".

const BUNNY_HOST = 'quan-images.b-cdn.net';
const FALLBACK_HOST = 'img.quan.hoabinh.vn';

function fixImage(img) {
  if (img.dataset.fallbackHostApplied) return;
  const url = new URL(img.currentSrc || img.src, window.location.href);
  if (url.hostname !== BUNNY_HOST) return;
  img.dataset.fallbackHostApplied = 'true';
  url.hostname = FALLBACK_HOST;
  img.src = url.toString();
}

function isArticleImage(img) {
  return img.closest('article') !== null;
}

document.addEventListener(
  'error',
  (event) => {
    if (event.target instanceof HTMLImageElement && isArticleImage(event.target)) {
      fixImage(event.target);
    }
  },
  true,
);

for (const img of document.images) {
  if (isArticleImage(img) && img.complete && img.naturalWidth === 0) {
    fixImage(img);
  }
}
