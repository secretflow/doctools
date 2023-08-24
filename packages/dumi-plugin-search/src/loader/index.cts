/* eslint-disable promise/no-callback-in-promise */
import type { LoaderDefinitionFunction } from 'webpack';

import type { LoaderOptions } from './typing.d.js';

const wrapper: LoaderDefinitionFunction<LoaderOptions> = function () {
  const options = this.getOptions();
  const callback = this.async();
  // FIXME: indicate dependencies
  // https://webpack.js.org/contribute/writing-a-loader/#loader-dependencies
  // https://webpack.js.org/contribute/writing-a-loader/#module-dependencies
  import('./index.mjs')
    .then(({ loader }) => loader(options))
    .then((content) => callback(null, content))
    .catch(callback);
};

module.exports = wrapper;
