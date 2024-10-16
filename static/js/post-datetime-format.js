import { minitz } from 'https://cdn.jsdelivr.net/gh/hexagon/minitz@4.0.6/dist/minitz.min.mjs'

document.addEventListener('alpine:init', () => {
  const shortFormatter = new Intl.DateTimeFormat(window.LANG, { dateStyle: 'medium' });
  const longFormatter = new Intl.DateTimeFormat(window.LANG, { dateStyle: 'long', timeStyle: 'full'});
  Alpine.data('post_meta', (created_at = null) => ({
    created_at: created_at ? minitz.fromTZISO(created_at) : null,
    get created_at_date_display() {
      return this.created_at ? shortFormatter.format(this.created_at) : ''
    },
    get created_at_full_display() {
      return this.created_at ? longFormatter.format(this.created_at) : ''
    },
  }));
})
