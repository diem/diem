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
const env_1 = require("./env");
const fs_extra_1 = __importDefault(require("fs-extra"));
const path_1 = __importDefault(require("path"));
const sidebars_1 = __importDefault(require("./sidebars"));
function docsVersion(version, siteDir, options) {
    if (!version) {
        throw new Error('No version tag specified!. Pass the version you wish to create as an argument. Ex: 1.0.0');
    }
    if (version.includes('/') || version.includes('\\')) {
        throw new Error(`Invalid version tag specified! Do not include slash (/) or (\\). Try something like: 1.0.0`);
    }
    if (version.length > 32) {
        throw new Error('Invalid version tag specified! Length must <= 32 characters. Try something like: 1.0.0');
    }
    // Since we are going to create `version-${version}` folder, we need to make
    // sure it's a valid pathname.
    if (/[<>:"\/\\|?*\x00-\x1F]/g.test(version)) {
        throw new Error('Invalid version tag specified! Please ensure its a valid pathname too. Try something like: 1.0.0');
    }
    if (/^\.\.?$/.test(version)) {
        throw new Error('Invalid version tag specified! Do not name your version "." or "..". Try something like: 1.0.0');
    }
    // Load existing versions.
    let versions = [];
    const versionsJSONFile = env_1.getVersionsJSONFile(siteDir);
    if (fs_extra_1.default.existsSync(versionsJSONFile)) {
        versions = JSON.parse(fs_extra_1.default.readFileSync(versionsJSONFile, 'utf8'));
    }
    // Check if version already exists.
    if (versions.includes(version)) {
        throw new Error('This version already exists!. Use a version tag that does not already exist.');
    }
    const { path: docsPath, sidebarPath } = options;
    // Copy docs files.
    const docsDir = path_1.default.join(siteDir, docsPath);
    if (fs_extra_1.default.existsSync(docsDir) && fs_extra_1.default.readdirSync(docsDir).length > 0) {
        const versionedDir = env_1.getVersionedDocsDir(siteDir);
        const newVersionDir = path_1.default.join(versionedDir, `version-${version}`);
        fs_extra_1.default.copySync(docsDir, newVersionDir);
    }
    else {
        throw new Error('There is no docs to version !');
    }
    // Load current sidebar and create a new versioned sidebars file.
    if (fs_extra_1.default.existsSync(sidebarPath)) {
        const loadedSidebars = sidebars_1.default([sidebarPath]);
        // Transform id in original sidebar to versioned id.
        const normalizeItem = (item) => {
            switch (item.type) {
                case 'category':
                    return Object.assign(Object.assign({}, item), { items: item.items.map(normalizeItem) });
                case 'ref':
                case 'doc':
                    return {
                        type: item.type,
                        id: `version-${version}/${item.id}`,
                    };
                default:
                    return item;
            }
        };
        const versionedSidebar = Object.entries(loadedSidebars).reduce((acc, [sidebarId, sidebarItems]) => {
            const newVersionedSidebarId = `version-${version}/${sidebarId}`;
            acc[newVersionedSidebarId] = sidebarItems.map(normalizeItem);
            return acc;
        }, {});
        const versionedSidebarsDir = env_1.getVersionedSidebarsDir(siteDir);
        const newSidebarFile = path_1.default.join(versionedSidebarsDir, `version-${version}-sidebars.json`);
        fs_extra_1.default.ensureDirSync(path_1.default.dirname(newSidebarFile));
        fs_extra_1.default.writeFileSync(newSidebarFile, `${JSON.stringify(versionedSidebar, null, 2)}\n`, 'utf8');
    }
    // Update versions.json file.
    versions.unshift(version);
    fs_extra_1.default.ensureDirSync(path_1.default.dirname(versionsJSONFile));
    fs_extra_1.default.writeFileSync(versionsJSONFile, `${JSON.stringify(versions, null, 2)}\n`);
    console.log(`Version ${version} created!`);
}
exports.docsVersion = docsVersion;
