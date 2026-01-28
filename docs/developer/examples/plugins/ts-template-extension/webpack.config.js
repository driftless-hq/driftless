const path = require('path');

module.exports = {
    entry: './dist/index.js',
    output: {
        path: path.resolve(__dirname, 'dist'),
        filename: 'driftless-ts-template-extension-plugin.js',
        library: {
            type: 'module'
        }
    },
    experiments: {
        outputModule: true
    },
    mode: 'production',
    optimization: {
        minimize: true
    }
};