const path = require('path');
const CopyWebpackPlugin = require('copy-webpack-plugin');
const WasmPackPlugin = require('@wasm-tool/wasm-pack-plugin');

const distPath = path.resolve(__dirname, "dist");
module.exports = (env, argv) => {
    return {
        entry: './bootstrap.js',
        output: {
            path: distPath,
            filename: "commits-loupe.js",
            webassemblyModuleFilename: "commits-loupe.wasm"
        },
        devServer: {
            contentBase: distPath,
            compress: argv.mode === 'production',
            port: 8000
        },
        plugins: [
            new CopyWebpackPlugin({
                patterns: [
                    { from: './static', to: distPath }
                ]
            }),
            new WasmPackPlugin({
                crateDirectory: ".",
                extraArgs: "--no-typescript",
            })
        ],
        watch: argv.mode !== 'production'
    };
};
