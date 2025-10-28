import { createHighlighter } from 'https://esm.sh/shiki@3.14.0'

let edgeQlGrammar = null

fetch('/static/js/edgeql.json')
	.then((res) => res.json())
	.then((o) => {
		edgeQlGrammar = o
	})

const LANGS = [
	'html',
	'css',
	'js',
	'typescript',
	'vue',
	'vue-html',
	'ts-tags',
	'astro',
	'svelte',
	'tsx',
	'xml',
	'python',
	'rust',
	'shellscript',
	'shellsession',
	'fish',
	'nushell',
	'c',
	'cpp',
	'go',
	'latex',
	'lua',
	'zig',
	'gleam',
	'dart',
	'elm',
	'json',
	'plsql',
	'sql',
	'desktop',
	'docker',
	'regexp',
	'rst',
	'md',
	'typst',
	'wikitext',
	'asciidoc',
	'fluent',
	'jinja',
	'qml',
	'ini',
	'toml',
	'ssh-config',
	'nginx',
	'systemd',
	'http',
	'log',
	'ansi',
	'typespec',
]

const taskShiki = createHighlighter({
	langs: LANGS,
	langAlias: {
		edgeql: 'EdgeQL',
	},
	themes: ['one-dark-pro'],
})

function delay() {
	return new Promise((resolve) => setTimeout(resolve, 100))
}

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
					for (const c of classes) {
						this.addClassToHast(node, c)
					}
					const style = node.properties.style || ''
					node.properties.style = `${style}--start-line: ${startLine};`
				},
				pre(node) {
					const className = classes.includes('q-with-lineno') ? 'py-4' : 'p-4'
					this.addClassToHast(node, className)
				},
			},
		],
	}
}

document.addEventListener('alpine:init', () => {
	Alpine.data('need_highlight', () => ({
		code: '',
		origClasses: [],
		startLine: 1,
		lang: 'text',
		init() {
			const codeElm = this.$refs.orig_code
			const code = codeElm.textContent
			this.code = code.trim()
			const classes = Array.from(codeElm.classList.values())
			const className = classes.find((c) => c.startsWith('language-'))
			if (className) {
				// From 'language-vue-html' take 'vue-html'
				this.lang = className.slice(9)
			}
			this.origClasses = classes
			if (codeElm.dataset.startLine) {
				this.startLine = Number.parseInt(codeElm.dataset.startLine)
			}
		},
		async highlight() {
			const lang = this.lang
			const classes = this.origClasses
			const opts = getShikiOpt(lang, classes, this.startLine)
			const highlighter = await taskShiki
			if (!edgeQlGrammar) {
				for (let i = 0; i < 5; i++) {
					await delay()
					if (edgeQlGrammar) {
						break
					}
				}
			}
			if (edgeQlGrammar) {
				await highlighter.loadLanguage(edgeQlGrammar)
			} else {
				console.warn('EdgeQL grammar is not available!')
			}
			const html = await highlighter.codeToHtml(this.code, opts)
			return html
		},
	}))
})
