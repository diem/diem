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
const fs_extra_1 = __importDefault(require("fs-extra"));
const path_1 = __importDefault(require("path"));
const utils_1 = require("@docusaurus/utils");
const lastUpdate_1 = __importDefault(require("./lastUpdate"));
async function lastUpdated(filePath, options) {
    const { showLastUpdateAuthor, showLastUpdateTime } = options;
    if (showLastUpdateAuthor || showLastUpdateTime) {
        // Use fake data in dev for faster development.
        const fileLastUpdateData = process.env.NODE_ENV === 'production'
            ? await lastUpdate_1.default(filePath)
            : {
                author: 'Author',
                timestamp: 1539502055,
            };
        if (fileLastUpdateData) {
            const { author, timestamp } = fileLastUpdateData;
            return {
                lastUpdatedAt: showLastUpdateTime ? timestamp : undefined,
                lastUpdatedBy: showLastUpdateAuthor ? author : undefined,
            };
        }
    }
    return {};
}
async function processMetadata({ source, refDir, context, options, env, }) {
    const { routeBasePath, editUrl } = options;
    const { siteDir, baseUrl } = context;
    const { versioning } = env;
    const filePath = path_1.default.join(refDir, source);
    const fileStringPromise = fs_extra_1.default.readFile(filePath, 'utf-8');
    const lastUpdatedPromise = lastUpdated(filePath, options);
    let version;
    const dirName = path_1.default.dirname(source);
    if (versioning.enabled) {
        if (/^version-/.test(dirName)) {
            const inferredVersion = dirName
                .split('/', 1)
                .shift()
                .replace(/^version-/, '');
            if (inferredVersion && versioning.versions.includes(inferredVersion)) {
                version = inferredVersion;
            }
        }
        else {
            version = 'next';
        }
    }
    // The version portion of the url path. Eg: 'next', '1.0.0', and ''.
    const versionPath = version && version !== versioning.latestVersion ? version : '';
    const relativePath = path_1.default.relative(siteDir, filePath);
    const docsEditUrl = utils_1.getEditUrl(relativePath, editUrl);
    const { frontMatter = {}, excerpt } = utils_1.parse(await fileStringPromise);
    const { sidebar_label, custom_edit_url } = frontMatter;
    // Default base id is the file name.
    const baseID = frontMatter.id || path_1.default.basename(source, path_1.default.extname(source));
    if (baseID.includes('/')) {
        throw new Error('Document id cannot include "/".');
    }
    // Append subdirectory as part of id.
    const id = dirName !== '.' ? `${dirName}/${baseID}` : baseID;
    // Default title is the id.
    const title = frontMatter.title || baseID;
    const description = frontMatter.description || excerpt;
    // The last portion of the url path. Eg: 'foo/bar', 'bar'.
    const routePath = version && version !== 'next'
        ? id.replace(new RegExp(`^version-${version}/`), '')
        : id;
    const permalink = utils_1.normalizeUrl([
        baseUrl,
        routeBasePath,
        versionPath,
        routePath,
    ]);
    const { lastUpdatedAt, lastUpdatedBy } = await lastUpdatedPromise;
    // Assign all of object properties during instantiation (if possible) for
    // NodeJS optimization.
    // Adding properties to object after instantiation will cause hidden
    // class transitions.
    const metadata = {
        id,
        title,
        description,
        source: utils_1.aliasedSitePath(filePath, siteDir),
        permalink,
        editUrl: custom_edit_url !== undefined ? custom_edit_url : docsEditUrl,
        version,
        lastUpdatedBy,
        lastUpdatedAt,
        sidebar_label,
    };
    return metadata;
}
exports.default = processMetadata;
