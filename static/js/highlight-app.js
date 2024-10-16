import { codeToHtml } from 'https://esm.sh/shiki@1.22.0'

// Our old <code> element will be replaced by the one created by Shiki,
// we need to keep old class names and copy to the new one.
function getShikiOpt(lang, classes, startLine) {
  return {
    lang,
    theme: 'one-dark-pro',
    // Ref: https://shiki.style/guide/transformers#transformer-hooks
    transformers: [
      {
        code(node) {
          classes.forEach((c) => this.addClassToHast(node, c))
          const style = node.properties.style || ''
          node.properties.style = style + `--start-line: ${startLine};`
        },
        pre(node) {
          const className = classes.includes('q-with-lineno') ? 'py-4' : 'p-4'
          this.addClassToHast(node, className)
        },
      }
    ]
  }

}

document.addEventListener('alpine:init', () => {
  Alpine.data('need_highlight', () => ({
    code: '',
    origClasses: [],
    startLine: 1,
    lang: 'text',
    init() {
      const codeElm = this.$refs.orig_code;
      const code = codeElm.textContent
      this.code = code.trim()
      const classes = Array.from(codeElm.classList.values())
      const className = classes.find((c) => c.startsWith('language-'))
      if (className) {
        this.lang = className.split('-')[1]
      }
      this.origClasses = classes
      codeElm.dataset.startLine && (this.startLine = parseInt(codeElm.dataset.startLine))
    },
    async highlight() {
      const lang = this.lang
      const classes = this.origClasses
      const opts = getShikiOpt(lang, classes, this.startLine)
      const html = await codeToHtml(this.code, opts)
      return html
    }
  }))
})
