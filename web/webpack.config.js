const path = require('path');

module.exports = (env, argv) => {
    const isProduction = argv.mode === 'production';
    
    return {
        entry: './wasmGuardianLoader.js',
        output: {
            path: path.resolve(__dirname, 'dist'),
            filename: isProduction ? 'wasmGuardianLoader.min.js' : 'wasmGuardianLoader.bundle.js',
            library: {
                name: 'WasmGuardianLoader',
                type: 'umd',
                export: 'default'
            },
            globalObject: 'this',
            clean: true
        },
        mode: argv.mode || 'development',
        devtool: isProduction ? 'source-map' : 'eval-source-map',
        module: {
            rules: [
                {
                    test: /\.js$/,
                    exclude: /node_modules/,
                    use: {
                        loader: 'babel-loader',
                        options: {
                            presets: [
                                ['@babel/preset-env', {
                                    targets: {
                                        browsers: ['> 1%', 'last 2 versions', 'not dead']
                                    },
                                    modules: false
                                }]
                            ]
                        }
                    }
                }
            ]
        },
        resolve: {
            extensions: ['.js']
        },
        optimization: {
            minimize: isProduction,
            minimizer: [
                new (require('terser-webpack-plugin'))({
                    terserOptions: {
                        compress: {
                            drop_console: isProduction,
                            drop_debugger: isProduction
                        },
                        mangle: {
                            keep_classnames: true,
                            keep_fnames: true
                        },
                        format: {
                            comments: false
                        }
                    },
                    extractComments: false
                })
            ]
        },
        plugins: [
            new (require('webpack')).BannerPlugin({
                banner: `
RusWaCipher Web Runtime v${require('./package.json').version}
Copyright (c) 2024 RusWaCipher Project
Licensed under MIT License
Build: ${new Date().toISOString()}
                `.trim()
            })
        ],
        externals: {
            // Don't bundle these if they're available globally
            'crypto': 'crypto'
        },
        performance: {
            hints: isProduction ? 'warning' : false,
            maxEntrypointSize: 250000,
            maxAssetSize: 250000
        }
    };
};
