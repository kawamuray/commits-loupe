const path = require('path');
const CopyWebpackPlugin = require('copy-webpack-plugin');
const WasmPackPlugin = require('@wasm-tool/wasm-pack-plugin');

const distPath = path.resolve(__dirname, "dist");

const mainConfig = (env, argv) => {
    return {
        entry: './bootstrap.js',
        output: {
            path: distPath,
            // publicPath: "commits-loupe/",
            filename: "commits-loupe.js",
            webassemblyModuleFilename: "commits-loupe.wasm",
            library: 'commitsLoupe'
        },
        devServer: {
            contentBase: distPath,
            compress: argv.mode === 'production',
            port: 8000
        },
        plugins: [
            new WasmPackPlugin({
                crateDirectory: ".",
                extraArgs: "--no-typescript",
            })
        ],
        watch: argv.mode !== 'production'
    };
};

const styleConfig = {
    entry: './style.js',
    output: {
        path: distPath,
        filename: "commits-loupe-style.js",
        library: 'commitsLoupeStyle'
    },
    module: {
        rules: [
            {
                test: /\.css$/i,
                use: ['style-loader', 'css-loader'],
            },
        ],
    },
    plugins: [
        new CopyWebpackPlugin({
            patterns: [
                { from: './static', to: distPath }
            ]
        }),
    ],
};

module.exports = [mainConfig, styleConfig];
