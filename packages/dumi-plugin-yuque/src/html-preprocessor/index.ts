/**
 * Replace all new lines (mostly from <pre />s) with &#10;, so that the MDX parser
 * doesn't treat them as new paragraphs and such.
 */
const loader = function (source: string) {
  // very hacky
  const htmlStart = source.search(/<!DOCTYPE html>/i);
  const frontmatter = source.slice(0, htmlStart);
  return frontmatter + source.slice(htmlStart).replace(/\n/g, '&#10;');
};

export default loader;
