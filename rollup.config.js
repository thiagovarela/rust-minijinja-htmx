
import resolve from '@rollup/plugin-node-resolve'
import esbuild from 'rollup-plugin-esbuild'
import replace from '@rollup/plugin-replace'

export default [
	{
		input: 'tiptap.js',
		output: {
			file: 'public/assets/tiptap.min.js',
			format: 'es'
		},
        plugins: [resolve(), replace({
			'process.env.NODE_ENV': JSON.stringify('production'),
		  }), esbuild({minify: true})]
	},
];