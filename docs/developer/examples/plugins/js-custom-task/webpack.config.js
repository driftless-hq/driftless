const path = require('path');

module.exports = {
    entry: './index.js',
    output: {
        path: path.resolve(__dirname, 'dist'),
        filename: 'driftless-js-custom-task-plugin.js',
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