"use strict";
/**
 * Copyright (c) Facebook, Inc. and its affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */
var __importDefault = (this && this.__importDefault) || function (mod) {
    return (mod && mod.__esModule) ? mod : { "default": mod };
};
const loader_utils_1 = require("loader-utils");
const linkify_1 = __importDefault(require("./linkify"));
module.exports = function (fileString) {
    const callback = this.async();
    const { docsDir, siteDir, versionedDir, sourceToPermalink } = loader_utils_1.getOptions(this);
    return (callback &&
        callback(null, linkify_1.default(fileString, this.resourcePath, docsDir, siteDir, sourceToPermalink, versionedDir)));
};
