const path = require('path');
const webpack = require('webpack');
const HtmlWebpackPlugin = require("html-webpack-plugin");

const PROD = 'production';
const DEV = 'development';

const mode = process.env.NODE_ENV === PROD ? PROD : DEV;

module.exports = {
    entry: './src/main.js',
    mode,
    output: {
        path: path.resolve(__dirname, 'dist'),
        filename: 'main.js'
    },
    plugins: [
        ...(mode === PROD ? [new webpack.NormalModuleReplacementPlugin(/env\/env.js/, './production.js')] : []),
        new HtmlWebpackPlugin({
            title: 'My App',
            filename: 'index.html',
            template: 'src/index.html',
            // minify: true
        }),
    ],
    devServer: {
        static: {
            directory: path.join(__dirname, 'dist'),
            watch: true,
        },
        port: !!process.env.PORT ? process.env.PORT : 9000,
        hot: true,
        liveReload: true
    }
};
