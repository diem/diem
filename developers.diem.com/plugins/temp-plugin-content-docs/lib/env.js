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
Object.defineProperty(exports, "__esModule", { value: true });
const path_1 = __importDefault(require("path"));
const fs_extra_1 = __importDefault(require("fs-extra"));
const constants_1 = require("./constants");
function getVersionedDocsDir(siteDir) {
    return path_1.default.join(siteDir, constants_1.VERSIONED_DOCS_DIR);
}
exports.getVersionedDocsDir = getVersionedDocsDir;
function getVersionedSidebarsDir(siteDir) {
    return path_1.default.join(siteDir, constants_1.VERSIONED_SIDEBARS_DIR);
}
exports.getVersionedSidebarsDir = getVersionedSidebarsDir;
function getVersionsJSONFile(siteDir) {
    return path_1.default.join(siteDir, constants_1.VERSIONS_JSON_FILE);
}
exports.getVersionsJSONFile = getVersionsJSONFile;
function default_1(siteDir) {
    const versioning = {
        enabled: false,
        versions: [],
        latestVersion: null,
        docsDir: '',
        sidebarsDir: '',
    };
    const versionsJSONFile = getVersionsJSONFile(siteDir);
    if (fs_extra_1.default.existsSync(versionsJSONFile)) {
        const parsedVersions = JSON.parse(fs_extra_1.default.readFileSync(versionsJSONFile, 'utf8'));
        if (parsedVersions && parsedVersions.length > 0) {
            versioning.latestVersion = parsedVersions[0];
            versioning.enabled = true;
            versioning.versions = parsedVersions;
            versioning.docsDir = getVersionedDocsDir(siteDir);
            versioning.sidebarsDir = getVersionedSidebarsDir(siteDir);
        }
    }
    return {
        versioning,
    };
}
exports.default = default_1;
