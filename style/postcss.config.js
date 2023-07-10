module.exports = {
    plugins: [
        require('cssnano')({
            preset: 'default',
        }),
        require('autoprefixer')(),
        require('postcss-import')(),
    ]
}