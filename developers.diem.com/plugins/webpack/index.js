const path = require('path');

module.exports = function (context, options) {
  return {
    name: 'custom-webpack-plugin',
    configureWebpack(config, isServer, utils) {
      const {getCacheLoader} = utils;
      return {
        resolve: {
          alias: {
            CSS: path.resolve(__dirname, '../../src/css'),
            components: path.resolve(__dirname, '../../src/components'),
            'react-axe': require.resolve("@axe-core/react"),
            'diem-docusaurus-components': path.resolve(
              __dirname,
              '../../src/@libra-opensource/diem-docusaurus-components',
            ),
            docs: path.resolve(__dirname, '../../docs'),
            img: path.resolve(__dirname, '../../static/img'),
            react: path.resolve('./node_modules/react'),
            src: path.resolve(__dirname, '../../src'),
          },
          fallback: {
            fs: false,
            http:   false, // require.resolve("stream-http"),
            https:  false, // require.resolve("https-browserify"),
            path:   false, // require.resolve("path-browserify"),
            crypto: false, // require.resolve("crypto-browserify"),
            stream: false, // require.resolve("stream-browserify"),
          },
        },
        node: {},
      };
    },
  };
};
