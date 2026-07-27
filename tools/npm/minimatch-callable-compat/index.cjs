"use strict";

const modern = require("minimatch-modern");

const callable = modern.minimatch;
Object.assign(callable, modern);

module.exports = callable;
