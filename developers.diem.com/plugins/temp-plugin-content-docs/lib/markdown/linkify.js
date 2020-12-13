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
const url_1 = require("url");
const utils_1 = require("@docusaurus/utils");
function default_1(fileString, filePath, docsDir, siteDir, sourceToPermalink, versionedDir) {
    // Determine the source dir. e.g: /website/docs, /website/versioned_docs/version-1.0.0
    let sourceDir;
    const thisSource = filePath;
    if (thisSource.startsWith(docsDir)) {
        sourceDir = docsDir;
    }
    else if (versionedDir && thisSource.startsWith(versionedDir)) {
        const specificVersionDir = utils_1.getSubFolder(thisSource, versionedDir);
        // e.g: specificVersionDir = version-1.0.0
        if (specificVersionDir) {
            sourceDir = path_1.default.join(versionedDir, specificVersionDir);
        }
    }
    let content = fileString;
    // Replace internal markdown linking (except in fenced blocks).
    if (sourceDir) {
        let fencedBlock = false;
        const lines = content.split('\n').map((line) => {
            if (line.trim().startsWith('```')) {
                fencedBlock = !fencedBlock;
            }
            if (fencedBlock)
                return line;
            let modifiedLine = line;
            // Replace inline-style links or reference-style links e.g:
            // This is [Document 1](doc1.md) -> we replace this doc1.md with correct link
            // [doc1]: doc1.md -> we replace this doc1.md with correct link
            const mdRegex = /(?:(?:\]\()|(?:\]:\s?))(?!https)([^'")\]\s>]+\.mdx?)/g;
            let mdMatch = mdRegex.exec(modifiedLine);
            while (mdMatch !== null) {
                // Replace it to correct html link.
                const mdLink = mdMatch[1];
                const targetSource = `${sourceDir}/${mdLink}`;
                const aliasedSource = (source) => `@site/${path_1.default.relative(siteDir, source)}`;
                const permalink = sourceToPermalink[aliasedSource(url_1.resolve(thisSource, mdLink))] ||
                    sourceToPermalink[aliasedSource(targetSource)];
                if (permalink) {
                    modifiedLine = modifiedLine.replace(mdLink, permalink);
                }
                mdMatch = mdRegex.exec(modifiedLine);
            }
            return modifiedLine;
        });
        content = lines.join('\n');
    }
    return content;
}
exports.default = default_1;
