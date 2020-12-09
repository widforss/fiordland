const path = require('path');

module.exports = {
  entry: './www/ts/main.ts',
  module: {
    rules: [
      {
        test: /\.tsx?$/,
        use: 'ts-loader',
        exclude: /node_modules/,
      },
    ],
  },
  resolve: {
    extensions: [ '.tsx', '.ts', '.js' ],
  },
  output: {
    filename: './www/static/js/fiordland.js',
    path: path.resolve(__dirname),
  },
};
