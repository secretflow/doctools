// @ts-check

import * as process from "node:process";

import { html, svg } from "property-information";

process.stdout.write(JSON.stringify({ ...svg.normal, ...html.normal }, null, 2));
