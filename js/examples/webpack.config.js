const path = require('path');
const { RuswacipherPlugin } = require('ruswacipher');

module.exports = {
  mode: 'production',
  entry: './src/index.js',
  output: {
    path: path.resolve(__dirname, 'dist'),
    filename: 'bundle.js',
  },
  module: {
    rules: [
      // Use RusWaCipher as the loader for WASM files
      {
        test: /\.wasm$/,
        type: 'webassembly/async',
        use: [
          {
            loader: 'wasm-loader'
          },
          {
            // RusWaCipher Loader - Apply obfuscation when loading WASM
            loader: 'ruswacipher/dist/webpack-loader',
            options: {
              level: 'high', // Optional values: 'low', 'medium', 'high' or 0, 1, 2
              allowOriginalLoader: false // If true, allow input to be a string
            }
          }
        ]
      }
    ]
  },
  plugins: [
    // RusWaCipher plugin - Use alternative method to obfuscate WASM files during build
    new RuswacipherPlugin({
      level: 'high', // Obfuscation level
      include: /\.wasm$/, // File matching pattern to obfuscate
      exclude: /node_modules/, // File matching pattern to exclude
      verbose: true // Enable verbose logging
    })
  ],
  experiments: {
    // Enable WebAssembly support
    asyncWebAssembly: true
  }
}; 