import typescript from '@rollup/plugin-typescript';
import {uglify} from 'rollup-plugin-uglify';

export default [{
    input: 'src/script/index.ts',
    output: {
        file: '../script.js',
        format: 'iife',
        sourcemap: false
    },
    plugins: [
        typescript(),
        uglify()
    ]
}];