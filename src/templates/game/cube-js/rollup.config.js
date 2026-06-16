import { nodeResolve } from '@rollup/plugin-node-resolve'
import commonjs from '@rollup/plugin-commonjs'
import jumpjet from '@jumpjet-runtime/rollup-plugin-jumpjet'

export default {
	input: 'game.js',
	output: {
		file: 'dist/game.js',
		format: 'esm'
	},
	plugins: [
		commonjs(), 
		nodeResolve(),
		jumpjet()
	]
}
