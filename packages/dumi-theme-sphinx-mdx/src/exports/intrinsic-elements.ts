import { highlighted } from '~/internals/common/highlighting.js';
import { permalink } from '~/internals/common/permalink.js';

const h1 = permalink(highlighted('h1'));
const h2 = permalink(highlighted('h2'));
const h3 = permalink(highlighted('h3'));
const h4 = permalink(highlighted('h4'));
const h5 = permalink(highlighted('h5'));
const h6 = permalink(highlighted('h6'));
const a = highlighted('a');

export { h1, h2, h3, h4, h5, h6, a };
