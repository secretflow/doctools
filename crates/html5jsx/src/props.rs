use swc_core::{atoms::Atom, ecma::ast::Lit};
use swc_html_ast::Attribute;

fn create_value<F>(atom: &Option<Atom>, from: F, default: Lit) -> Option<Lit>
where
  F: FnOnce(&Atom) -> Option<Lit>,
{
  match atom {
    Some(atom) => from(atom),
    None => Some(default),
  }
}

fn as_string(atom: &Atom) -> Option<Lit> {
  Some(atom.as_str().into())
}

fn as_boolean(atom: &Atom) -> Option<Lit> {
  match atom.as_str() {
    "false" => Some(false.into()),
    _ => Some(true.into()),
  }
}

fn as_overloaded_boolean(atom: &Atom) -> Option<Lit> {
  match atom.as_str() {
    "false" => Some(false.into()),
    "true" => Some(true.into()),
    _ => as_string(atom),
  }
}

fn as_number(atom: &Atom) -> Option<Lit> {
  match atom.as_str().parse::<f64>() {
    Ok(v) => Some(v.into()),
    Err(_) => None,
  }
}

fn reject_unsafe_inline_javascript(atom: &Atom) -> Option<Lit> {
  if atom.as_str().contains("javascript:") {
    if cfg!(feature = "unsafe-ignore") {
      None
    } else if cfg!(feature = "unsafe-ignore") {
      as_string(atom)
    } else {
      panic!("refuse to convert `javascript:` URLs")
    }
  } else {
    as_string(atom)
  }
}

pub fn convert_attribute(attr: &Attribute) -> Option<(Lit, Lit)> {
  if cfg!(feature = "unsafe-ignore") {
    if attr.name.to_lowercase() == "dangerouslysetinnerhtml" || attr.name.starts_with("on") {
      return None;
    }
  } else if cfg!(feature = "unsafe-allow") {
    ();
  } else {
    if attr.name.to_lowercase() == "dangerouslysetinnerhtml" {
      panic!("refuse to convert dangerouslySetInnerHTML")
    }
    if attr.name.starts_with("on") {
      panic!("refuse to convert event handlers")
    }
  }

  match attr.name.as_str() {
    "children" | "ref" | "key" | "class" | "style" => return None,
    _ => (),
  };

  // https://github.com/facebook/react/blob/v18.2.0/packages/react-dom/src/shared/possibleStandardNames.js
  let name = match attr.name.as_str() {
    "accept" => "accept",
    "acceptcharset" => "acceptCharset",
    "accept-charset" => "acceptCharset",
    "accesskey" => "accessKey",
    "action" => "action",
    "allowfullscreen" => "allowFullScreen",
    "alt" => "alt",
    "as" => "as",
    "async" => "async",
    "autocapitalize" => "autoCapitalize",
    "autocomplete" => "autoComplete",
    "autocorrect" => "autoCorrect",
    "autofocus" => "autoFocus",
    "autoplay" => "autoPlay",
    "autosave" => "autoSave",
    "capture" => "capture",
    "cellpadding" => "cellPadding",
    "cellspacing" => "cellSpacing",
    "challenge" => "challenge",
    "charset" => "charSet",
    "checked" => "checked",
    "children" => "children",
    "cite" => "cite",
    "class" => "className",
    "classid" => "classID",
    "classname" => "className",
    "cols" => "cols",
    "colspan" => "colSpan",
    "content" => "content",
    "contenteditable" => "contentEditable",
    "contextmenu" => "contextMenu",
    "controls" => "controls",
    "controlslist" => "controlsList",
    "coords" => "coords",
    "crossorigin" => "crossOrigin",
    "dangerouslysetinnerhtml" => "dangerouslySetInnerHTML",
    "data" => "data",
    "datetime" => "dateTime",
    "default" => "default",
    "defaultchecked" => "defaultChecked",
    "defaultvalue" => "defaultValue",
    "defer" => "defer",
    "dir" => "dir",
    "disabled" => "disabled",
    "disablepictureinpicture" => "disablePictureInPicture",
    "disableremoteplayback" => "disableRemotePlayback",
    "download" => "download",
    "draggable" => "draggable",
    "enctype" => "encType",
    "enterkeyhint" => "enterKeyHint",
    "for" => "htmlFor",
    "form" => "form",
    "formmethod" => "formMethod",
    "formaction" => "formAction",
    "formenctype" => "formEncType",
    "formnovalidate" => "formNoValidate",
    "formtarget" => "formTarget",
    "frameborder" => "frameBorder",
    "headers" => "headers",
    "height" => "height",
    "hidden" => "hidden",
    "high" => "high",
    "href" => "href",
    "hreflang" => "hrefLang",
    "htmlfor" => "htmlFor",
    "httpequiv" => "httpEquiv",
    "http-equiv" => "httpEquiv",
    "icon" => "icon",
    "id" => "id",
    "imagesizes" => "imageSizes",
    "imagesrcset" => "imageSrcSet",
    "innerhtml" => "innerHTML",
    "inputmode" => "inputMode",
    "integrity" => "integrity",
    "is" => "is",
    "itemid" => "itemID",
    "itemprop" => "itemProp",
    "itemref" => "itemRef",
    "itemscope" => "itemScope",
    "itemtype" => "itemType",
    "keyparams" => "keyParams",
    "keytype" => "keyType",
    "kind" => "kind",
    "label" => "label",
    "lang" => "lang",
    "list" => "list",
    "loop" => "loop",
    "low" => "low",
    "manifest" => "manifest",
    "marginwidth" => "marginWidth",
    "marginheight" => "marginHeight",
    "max" => "max",
    "maxlength" => "maxLength",
    "media" => "media",
    "mediagroup" => "mediaGroup",
    "method" => "method",
    "min" => "min",
    "minlength" => "minLength",
    "multiple" => "multiple",
    "muted" => "muted",
    "name" => "name",
    "nomodule" => "noModule",
    "nonce" => "nonce",
    "novalidate" => "noValidate",
    "open" => "open",
    "optimum" => "optimum",
    "pattern" => "pattern",
    "placeholder" => "placeholder",
    "playsinline" => "playsInline",
    "poster" => "poster",
    "preload" => "preload",
    "profile" => "profile",
    "radiogroup" => "radioGroup",
    "readonly" => "readOnly",
    "referrerpolicy" => "referrerPolicy",
    "rel" => "rel",
    "required" => "required",
    "reversed" => "reversed",
    "role" => "role",
    "rows" => "rows",
    "rowspan" => "rowSpan",
    "sandbox" => "sandbox",
    "scope" => "scope",
    "scoped" => "scoped",
    "scrolling" => "scrolling",
    "seamless" => "seamless",
    "selected" => "selected",
    "shape" => "shape",
    "size" => "size",
    "sizes" => "sizes",
    "span" => "span",
    "spellcheck" => "spellCheck",
    "src" => "src",
    "srcdoc" => "srcDoc",
    "srclang" => "srcLang",
    "srcset" => "srcSet",
    "start" => "start",
    "step" => "step",
    "style" => "style",
    "summary" => "summary",
    "tabindex" => "tabIndex",
    "target" => "target",
    "title" => "title",
    "type" => "type",
    "usemap" => "useMap",
    "value" => "value",
    "width" => "width",
    "wmode" => "wmode",
    "wrap" => "wrap",
    "about" => "about",
    "accentheight" => "accentHeight",
    "accent-height" => "accentHeight",
    "accumulate" => "accumulate",
    "additive" => "additive",
    "alignmentbaseline" => "alignmentBaseline",
    "alignment-baseline" => "alignmentBaseline",
    "allowreorder" => "allowReorder",
    "alphabetic" => "alphabetic",
    "amplitude" => "amplitude",
    "arabicform" => "arabicForm",
    "arabic-form" => "arabicForm",
    "ascent" => "ascent",
    "attributename" => "attributeName",
    "attributetype" => "attributeType",
    "autoreverse" => "autoReverse",
    "azimuth" => "azimuth",
    "basefrequency" => "baseFrequency",
    "baselineshift" => "baselineShift",
    "baseline-shift" => "baselineShift",
    "baseprofile" => "baseProfile",
    "bbox" => "bbox",
    "begin" => "begin",
    "bias" => "bias",
    "by" => "by",
    "calcmode" => "calcMode",
    "capheight" => "capHeight",
    "cap-height" => "capHeight",
    "clip" => "clip",
    "clippath" => "clipPath",
    "clip-path" => "clipPath",
    "clippathunits" => "clipPathUnits",
    "cliprule" => "clipRule",
    "clip-rule" => "clipRule",
    "color" => "color",
    "colorinterpolation" => "colorInterpolation",
    "color-interpolation" => "colorInterpolation",
    "colorinterpolationfilters" => "colorInterpolationFilters",
    "color-interpolation-filters" => "colorInterpolationFilters",
    "colorprofile" => "colorProfile",
    "color-profile" => "colorProfile",
    "colorrendering" => "colorRendering",
    "color-rendering" => "colorRendering",
    "contentscripttype" => "contentScriptType",
    "contentstyletype" => "contentStyleType",
    "cursor" => "cursor",
    "cx" => "cx",
    "cy" => "cy",
    "d" => "d",
    "datatype" => "datatype",
    "decelerate" => "decelerate",
    "descent" => "descent",
    "diffuseconstant" => "diffuseConstant",
    "direction" => "direction",
    "display" => "display",
    "divisor" => "divisor",
    "dominantbaseline" => "dominantBaseline",
    "dominant-baseline" => "dominantBaseline",
    "dur" => "dur",
    "dx" => "dx",
    "dy" => "dy",
    "edgemode" => "edgeMode",
    "elevation" => "elevation",
    "enablebackground" => "enableBackground",
    "enable-background" => "enableBackground",
    "end" => "end",
    "exponent" => "exponent",
    "externalresourcesrequired" => "externalResourcesRequired",
    "fill" => "fill",
    "fillopacity" => "fillOpacity",
    "fill-opacity" => "fillOpacity",
    "fillrule" => "fillRule",
    "fill-rule" => "fillRule",
    "filter" => "filter",
    "filterres" => "filterRes",
    "filterunits" => "filterUnits",
    "floodopacity" => "floodOpacity",
    "flood-opacity" => "floodOpacity",
    "floodcolor" => "floodColor",
    "flood-color" => "floodColor",
    "focusable" => "focusable",
    "fontfamily" => "fontFamily",
    "font-family" => "fontFamily",
    "fontsize" => "fontSize",
    "font-size" => "fontSize",
    "fontsizeadjust" => "fontSizeAdjust",
    "font-size-adjust" => "fontSizeAdjust",
    "fontstretch" => "fontStretch",
    "font-stretch" => "fontStretch",
    "fontstyle" => "fontStyle",
    "font-style" => "fontStyle",
    "fontvariant" => "fontVariant",
    "font-variant" => "fontVariant",
    "fontweight" => "fontWeight",
    "font-weight" => "fontWeight",
    "format" => "format",
    "from" => "from",
    "fx" => "fx",
    "fy" => "fy",
    "g1" => "g1",
    "g2" => "g2",
    "glyphname" => "glyphName",
    "glyph-name" => "glyphName",
    "glyphorientationhorizontal" => "glyphOrientationHorizontal",
    "glyph-orientation-horizontal" => "glyphOrientationHorizontal",
    "glyphorientationvertical" => "glyphOrientationVertical",
    "glyph-orientation-vertical" => "glyphOrientationVertical",
    "glyphref" => "glyphRef",
    "gradienttransform" => "gradientTransform",
    "gradientunits" => "gradientUnits",
    "hanging" => "hanging",
    "horizadvx" => "horizAdvX",
    "horiz-adv-x" => "horizAdvX",
    "horizoriginx" => "horizOriginX",
    "horiz-origin-x" => "horizOriginX",
    "ideographic" => "ideographic",
    "imagerendering" => "imageRendering",
    "image-rendering" => "imageRendering",
    "in2" => "in2",
    "in" => "in",
    "inlist" => "inlist",
    "intercept" => "intercept",
    "k1" => "k1",
    "k2" => "k2",
    "k3" => "k3",
    "k4" => "k4",
    "k" => "k",
    "kernelmatrix" => "kernelMatrix",
    "kernelunitlength" => "kernelUnitLength",
    "kerning" => "kerning",
    "keypoints" => "keyPoints",
    "keysplines" => "keySplines",
    "keytimes" => "keyTimes",
    "lengthadjust" => "lengthAdjust",
    "letterspacing" => "letterSpacing",
    "letter-spacing" => "letterSpacing",
    "lightingcolor" => "lightingColor",
    "lighting-color" => "lightingColor",
    "limitingconeangle" => "limitingConeAngle",
    "local" => "local",
    "markerend" => "markerEnd",
    "marker-end" => "markerEnd",
    "markerheight" => "markerHeight",
    "markermid" => "markerMid",
    "marker-mid" => "markerMid",
    "markerstart" => "markerStart",
    "marker-start" => "markerStart",
    "markerunits" => "markerUnits",
    "markerwidth" => "markerWidth",
    "mask" => "mask",
    "maskcontentunits" => "maskContentUnits",
    "maskunits" => "maskUnits",
    "mathematical" => "mathematical",
    "mode" => "mode",
    "numoctaves" => "numOctaves",
    "offset" => "offset",
    "opacity" => "opacity",
    "operator" => "operator",
    "order" => "order",
    "orient" => "orient",
    "orientation" => "orientation",
    "origin" => "origin",
    "overflow" => "overflow",
    "overlineposition" => "overlinePosition",
    "overline-position" => "overlinePosition",
    "overlinethickness" => "overlineThickness",
    "overline-thickness" => "overlineThickness",
    "paintorder" => "paintOrder",
    "paint-order" => "paintOrder",
    "panose1" => "panose1",
    "panose-1" => "panose1",
    "pathlength" => "pathLength",
    "patterncontentunits" => "patternContentUnits",
    "patterntransform" => "patternTransform",
    "patternunits" => "patternUnits",
    "pointerevents" => "pointerEvents",
    "pointer-events" => "pointerEvents",
    "points" => "points",
    "pointsatx" => "pointsAtX",
    "pointsaty" => "pointsAtY",
    "pointsatz" => "pointsAtZ",
    "prefix" => "prefix",
    "preservealpha" => "preserveAlpha",
    "preserveaspectratio" => "preserveAspectRatio",
    "primitiveunits" => "primitiveUnits",
    "property" => "property",
    "r" => "r",
    "radius" => "radius",
    "refx" => "refX",
    "refy" => "refY",
    "renderingintent" => "renderingIntent",
    "rendering-intent" => "renderingIntent",
    "repeatcount" => "repeatCount",
    "repeatdur" => "repeatDur",
    "requiredextensions" => "requiredExtensions",
    "requiredfeatures" => "requiredFeatures",
    "resource" => "resource",
    "restart" => "restart",
    "result" => "result",
    "results" => "results",
    "rotate" => "rotate",
    "rx" => "rx",
    "ry" => "ry",
    "scale" => "scale",
    "security" => "security",
    "seed" => "seed",
    "shaperendering" => "shapeRendering",
    "shape-rendering" => "shapeRendering",
    "slope" => "slope",
    "spacing" => "spacing",
    "specularconstant" => "specularConstant",
    "specularexponent" => "specularExponent",
    "speed" => "speed",
    "spreadmethod" => "spreadMethod",
    "startoffset" => "startOffset",
    "stddeviation" => "stdDeviation",
    "stemh" => "stemh",
    "stemv" => "stemv",
    "stitchtiles" => "stitchTiles",
    "stopcolor" => "stopColor",
    "stop-color" => "stopColor",
    "stopopacity" => "stopOpacity",
    "stop-opacity" => "stopOpacity",
    "strikethroughposition" => "strikethroughPosition",
    "strikethrough-position" => "strikethroughPosition",
    "strikethroughthickness" => "strikethroughThickness",
    "strikethrough-thickness" => "strikethroughThickness",
    "string" => "string",
    "stroke" => "stroke",
    "strokedasharray" => "strokeDasharray",
    "stroke-dasharray" => "strokeDasharray",
    "strokedashoffset" => "strokeDashoffset",
    "stroke-dashoffset" => "strokeDashoffset",
    "strokelinecap" => "strokeLinecap",
    "stroke-linecap" => "strokeLinecap",
    "strokelinejoin" => "strokeLinejoin",
    "stroke-linejoin" => "strokeLinejoin",
    "strokemiterlimit" => "strokeMiterlimit",
    "stroke-miterlimit" => "strokeMiterlimit",
    "strokewidth" => "strokeWidth",
    "stroke-width" => "strokeWidth",
    "strokeopacity" => "strokeOpacity",
    "stroke-opacity" => "strokeOpacity",
    "suppresscontenteditablewarning" => "suppressContentEditableWarning",
    "suppresshydrationwarning" => "suppressHydrationWarning",
    "surfacescale" => "surfaceScale",
    "systemlanguage" => "systemLanguage",
    "tablevalues" => "tableValues",
    "targetx" => "targetX",
    "targety" => "targetY",
    "textanchor" => "textAnchor",
    "text-anchor" => "textAnchor",
    "textdecoration" => "textDecoration",
    "text-decoration" => "textDecoration",
    "textlength" => "textLength",
    "textrendering" => "textRendering",
    "text-rendering" => "textRendering",
    "to" => "to",
    "transform" => "transform",
    "typeof" => "typeof",
    "u1" => "u1",
    "u2" => "u2",
    "underlineposition" => "underlinePosition",
    "underline-position" => "underlinePosition",
    "underlinethickness" => "underlineThickness",
    "underline-thickness" => "underlineThickness",
    "unicode" => "unicode",
    "unicodebidi" => "unicodeBidi",
    "unicode-bidi" => "unicodeBidi",
    "unicoderange" => "unicodeRange",
    "unicode-range" => "unicodeRange",
    "unitsperem" => "unitsPerEm",
    "units-per-em" => "unitsPerEm",
    "unselectable" => "unselectable",
    "valphabetic" => "vAlphabetic",
    "v-alphabetic" => "vAlphabetic",
    "values" => "values",
    "vectoreffect" => "vectorEffect",
    "vector-effect" => "vectorEffect",
    "version" => "version",
    "vertadvy" => "vertAdvY",
    "vert-adv-y" => "vertAdvY",
    "vertoriginx" => "vertOriginX",
    "vert-origin-x" => "vertOriginX",
    "vertoriginy" => "vertOriginY",
    "vert-origin-y" => "vertOriginY",
    "vhanging" => "vHanging",
    "v-hanging" => "vHanging",
    "videographic" => "vIdeographic",
    "v-ideographic" => "vIdeographic",
    "viewbox" => "viewBox",
    "viewtarget" => "viewTarget",
    "visibility" => "visibility",
    "vmathematical" => "vMathematical",
    "v-mathematical" => "vMathematical",
    "vocab" => "vocab",
    "widths" => "widths",
    "wordspacing" => "wordSpacing",
    "word-spacing" => "wordSpacing",
    "writingmode" => "writingMode",
    "writing-mode" => "writingMode",
    "x1" => "x1",
    "x2" => "x2",
    "x" => "x",
    "xchannelselector" => "xChannelSelector",
    "xheight" => "xHeight",
    "x-height" => "xHeight",
    "xlinkactuate" => "xlinkActuate",
    "xlink:actuate" => "xlinkActuate",
    "xlinkarcrole" => "xlinkArcrole",
    "xlink:arcrole" => "xlinkArcrole",
    "xlinkhref" => "xlinkHref",
    "xlink:href" => "xlinkHref",
    "xlinkrole" => "xlinkRole",
    "xlink:role" => "xlinkRole",
    "xlinkshow" => "xlinkShow",
    "xlink:show" => "xlinkShow",
    "xlinktitle" => "xlinkTitle",
    "xlink:title" => "xlinkTitle",
    "xlinktype" => "xlinkType",
    "xlink:type" => "xlinkType",
    "xmlbase" => "xmlBase",
    "xml:base" => "xmlBase",
    "xmllang" => "xmlLang",
    "xml:lang" => "xmlLang",
    "xmlns" => "xmlns",
    "xml:space" => "xmlSpace",
    "xmlnsxlink" => "xmlnsXlink",
    "xmlns:xlink" => "xmlnsXlink",
    "xmlspace" => "xmlSpace",
    "y1" => "y1",
    "y2" => "y2",
    "y" => "y",
    "ychannelselector" => "yChannelSelector",
    "z" => "z",
    "zoomandpan" => "zoomAndPan",
    _ => attr.name.as_str(),
  };

  macro_rules! coerce_value {
    ( $name:ident, $($prop:literal << [$convert:ident, $default:literal]),* ) => {
      match $name {
        $( $prop => {
          match create_value(&attr.value, $convert, $default.into()) {
            Some(value) => return Some(($name.into(), value)),
            None => return None,
          }
        }, )*
        _ => ()
      }
    };
  }

  // https://github.com/facebook/react/blob/v18.2.0/packages/react-dom/src/shared/DOMProperty.js
  coerce_value!(
    name,
    // booleanish
    "contentEditable" << [as_boolean, true],
    "draggable" << [as_boolean, true],
    "spellCheck" << [as_boolean, true],
    "value" << [as_string, true],
    "autoReverse" << [as_boolean, true],
    "externalResourcesRequired" << [as_boolean, true],
    "focusable" << [as_boolean, true],
    "preserveAlpha" << [as_boolean, true],
    // boolean
    "allowFullScreen" << [as_boolean, true],
    "async" << [as_boolean, true],
    "autoFocus" << [as_boolean, true],
    "autoPlay" << [as_boolean, true],
    "controls" << [as_boolean, true],
    "default" << [as_boolean, true],
    "defer" << [as_boolean, true],
    "disabled" << [as_boolean, true],
    "disablePictureInPicture" << [as_boolean, true],
    "disableRemotePlayback" << [as_boolean, true],
    "formNoValidate" << [as_boolean, true],
    "hidden" << [as_boolean, true],
    "loop" << [as_boolean, true],
    "noModule" << [as_boolean, true],
    "noValidate" << [as_boolean, true],
    "open" << [as_boolean, true],
    "playsInline" << [as_boolean, true],
    "readOnly" << [as_boolean, true],
    "required" << [as_boolean, true],
    "reversed" << [as_boolean, true],
    "scoped" << [as_boolean, true],
    "seamless" << [as_boolean, true],
    "itemScope" << [as_boolean, true],
    "checked" << [as_boolean, true],
    "multiple" << [as_boolean, true],
    "muted" << [as_boolean, true],
    "selected" << [as_boolean, true],
    // overloaded boolean
    "capture" << [as_overloaded_boolean, true],
    "download" << [as_overloaded_boolean, true],
    // numbers
    "cols" << [as_number, ""],
    "rows" << [as_number, ""],
    "size" << [as_number, ""],
    "span" << [as_number, ""],
    "rowSpan" << [as_number, ""],
    "start" << [as_number, ""],
    // URLs
    "xlinkHref" << [reject_unsafe_inline_javascript, ""],
    "href" << [reject_unsafe_inline_javascript, ""],
    "src" << [reject_unsafe_inline_javascript, ""],
    "action" << [reject_unsafe_inline_javascript, ""],
    "formAction" << [reject_unsafe_inline_javascript, ""],
    // string
    "acceptCharset" << [as_string, ""],
    "htmlFor" << [as_string, ""],
    "httpEquiv" << [as_string, ""],
    "tabIndex" << [as_string, ""],
    "crossOrigin" << [as_string, ""],
    "accentHeight" << [as_string, ""],
    "alignmentBaseline" << [as_string, ""],
    "arabicForm" << [as_string, ""],
    "baselineShift" << [as_string, ""],
    "capHeight" << [as_string, ""],
    "clipPath" << [as_string, ""],
    "clipRule" << [as_string, ""],
    "colorInterpolation" << [as_string, ""],
    "colorInterpolationFilters" << [as_string, ""],
    "colorProfile" << [as_string, ""],
    "colorRendering" << [as_string, ""],
    "dominantBaseline" << [as_string, ""],
    "enableBackground" << [as_string, ""],
    "fillOpacity" << [as_string, ""],
    "fillRule" << [as_string, ""],
    "floodColor" << [as_string, ""],
    "floodOpacity" << [as_string, ""],
    "fontFamily" << [as_string, ""],
    "fontSize" << [as_string, ""],
    "fontSizeAdjust" << [as_string, ""],
    "fontStretch" << [as_string, ""],
    "fontStyle" << [as_string, ""],
    "fontVariant" << [as_string, ""],
    "fontWeight" << [as_string, ""],
    "glyphName" << [as_string, ""],
    "glyphOrientationHorizontal" << [as_string, ""],
    "glyphOrientationVertical" << [as_string, ""],
    "horizAdvX" << [as_string, ""],
    "horizOriginX" << [as_string, ""],
    "imageRendering" << [as_string, ""],
    "letterSpacing" << [as_string, ""],
    "lightingColor" << [as_string, ""],
    "markerEnd" << [as_string, ""],
    "markerMid" << [as_string, ""],
    "markerStart" << [as_string, ""],
    "overlinePosition" << [as_string, ""],
    "overlineThickness" << [as_string, ""],
    "paintOrder" << [as_string, ""],
    "panose-1" << [as_string, ""],
    "pointerEvents" << [as_string, ""],
    "renderingIntent" << [as_string, ""],
    "shapeRendering" << [as_string, ""],
    "stopColor" << [as_string, ""],
    "stopOpacity" << [as_string, ""],
    "strikethroughPosition" << [as_string, ""],
    "strikethroughThickness" << [as_string, ""],
    "strokeDasharray" << [as_string, ""],
    "strokeDashoffset" << [as_string, ""],
    "strokeLinecap" << [as_string, ""],
    "strokeLinejoin" << [as_string, ""],
    "strokeMiterlimit" << [as_string, ""],
    "strokeOpacity" << [as_string, ""],
    "strokeWidth" << [as_string, ""],
    "textAnchor" << [as_string, ""],
    "textDecoration" << [as_string, ""],
    "textRendering" << [as_string, ""],
    "underlinePosition" << [as_string, ""],
    "underlineThickness" << [as_string, ""],
    "unicodeBidi" << [as_string, ""],
    "unicodeRange" << [as_string, ""],
    "unitsPerEm" << [as_string, ""],
    "vAlphabetic" << [as_string, ""],
    "vHanging" << [as_string, ""],
    "vIdeographic" << [as_string, ""],
    "vMathematical" << [as_string, ""],
    "vectorEffect" << [as_string, ""],
    "vertAdvY" << [as_string, ""],
    "vertOriginX" << [as_string, ""],
    "vertOriginY" << [as_string, ""],
    "wordSpacing" << [as_string, ""],
    "writingMode" << [as_string, ""],
    "xmlnsXlink" << [as_string, ""],
    "xHeight" << [as_string, ""],
    "xlinkActuate" << [as_string, ""],
    "xlinkArcrole" << [as_string, ""],
    "xlinkRole" << [as_string, ""],
    "xlinkShow" << [as_string, ""],
    "xlinkTitle" << [as_string, ""],
    "xlinkType" << [as_string, ""],
    "xmlBase" << [as_string, ""],
    "xmlLang" << [as_string, ""],
    "xmlSpace" << [as_string, ""]
  );

  match create_value(&attr.value, as_string, true.into()) {
    Some(value) => Some((name.into(), value)),
    None => None,
  }
}
