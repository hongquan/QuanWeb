import { defineConfig } from 'unocss'

export default defineConfig({
	cli: {
		entry: {
			patterns: ['./minijinja/**/*.jinja', './src/**/*.rs', './static/js/*.js'],
			details: true,
			// outFile: 'static/css/uno.css',
		},
	},
	preflights: false,
})
