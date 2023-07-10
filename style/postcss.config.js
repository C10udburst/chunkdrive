module.exports = {
    plugins: [
        require('postcss-focus')(),
        require('postcss-simple-vars')(),
        require('autoprefixer')(),
        require('postcss-import')(),
        require('postcss-import-url')(),
        require('postcss-variable-compress')(),
        require('cssnano')({
            preset: 'default',
        }),
        require('postcss-font-magician')()
    ]
}