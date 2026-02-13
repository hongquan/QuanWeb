import { minitz } from 'https://cdn.jsdelivr.net/gh/hexagon/minitz@4.0.6/dist/minitz.min.mjs'

document.addEventListener('alpine:init', () => {
  const shortFormatter = new Intl.DateTimeFormat(window.LANG, { dateStyle: 'medium' });

  Alpine.data('featured_post_item', () => ({
    hovered: false,
    createdAt: null,
    init() {
      const createdAtStr = this.$el.dataset.createdAt;
      if (createdAtStr) {
        this.createdAt = minitz.fromTZISO(createdAtStr);
      }
    },
    get createdAtDisplay() {
      return this.createdAt ? shortFormatter.format(this.createdAt) : '';
    },
  }));
});
