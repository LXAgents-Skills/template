#!/usr/bin/env node

/*
 * Server entry point: read the configuration, then start the transport it names.
 *
 * Every module below this one is inert on import. This file is the only one that
 * starts anything, so everything else can be imported, and tested, without binding a
 * port or forking.
 */

import { loadConfig } from "./config.js";
import { run } from "./run.js";

await run(loadConfig());
