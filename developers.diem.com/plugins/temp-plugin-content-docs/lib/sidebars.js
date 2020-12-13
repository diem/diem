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
const lodash_flatmap_1 = __importDefault(require("lodash.flatmap"));
const fs_extra_1 = __importDefault(require("fs-extra"));
const import_fresh_1 = __importDefault(require("import-fresh"));
function isCategoryShorthand(item) {
    return typeof item !== 'string' && !item.type;
}
/**
 * Convert {category1: [item1,item2]} shorthand syntax to long-form syntax
 */
function normalizeCategoryShorthand(sidebar) {
    return Object.entries(sidebar).map(([label, items]) => ({
        type: 'category',
        label,
        items,
    }));
}
/**
 * Check that item contains only allowed keys.
 */
function assertItem(item, keys) {
    const unknownKeys = Object.keys(item).filter((key) => !keys.includes(key) && key !== 'type' && key !== 'extra');
    if (unknownKeys.length) {
        throw new Error(`Unknown sidebar item keys: ${unknownKeys}. Item: ${JSON.stringify(item)}`);
    }
}
function assertIsCategory(item) {
    assertItem(item, ['items', 'label']);
    if (typeof item.label !== 'string') {
        throw new Error(`Error loading ${JSON.stringify(item)}. "label" must be a string.`);
    }
    if (!Array.isArray(item.items)) {
        throw new Error(`Error loading ${JSON.stringify(item)}. "items" must be an array.`);
    }
}
function assertIsDoc(item) {
    assertItem(item, ['id']);
    if (typeof item.id !== 'string') {
        throw new Error(`Error loading ${JSON.stringify(item)}. "id" must be a string.`);
    }
}
function assertIsLink(item) {
    assertItem(item, ['href', 'label']);
    if (typeof item.href !== 'string') {
        throw new Error(`Error loading ${JSON.stringify(item)}. "href" must be a string.`);
    }
    if (typeof item.label !== 'string') {
        throw new Error(`Error loading ${JSON.stringify(item)}. "label" must be a string.`);
    }
}
/**
 * Normalizes recursively item and all its children. Ensures that at the end
 * each item will be an object with the corresponding type.
 */
function normalizeItem(item) {
    if (typeof item === 'string') {
        return [
            {
                type: 'doc',
                id: item,
            },
        ];
    }
    if (isCategoryShorthand(item)) {
        return lodash_flatmap_1.default(normalizeCategoryShorthand(item), normalizeItem);
    }
    switch (item.type) {
        case 'category':
            assertIsCategory(item);
            return [Object.assign(Object.assign({}, item), { items: lodash_flatmap_1.default(item.items, normalizeItem) })];
        case 'link':
            assertIsLink(item);
            return [item];
        case 'ref':
        case 'doc':
            assertIsDoc(item);
            return [item];
        default:
            throw new Error(`Unknown sidebar item type: ${item.type}`);
    }
}
/**
 * Converts sidebars object to mapping to arrays of sidebar item objects.
 */
function normalizeSidebar(sidebars) {
    return Object.entries(sidebars).reduce((acc, [sidebarId, sidebar]) => {
        const normalizedSidebar = Array.isArray(sidebar)
            ? sidebar
            : normalizeCategoryShorthand(sidebar);
        acc[sidebarId] = lodash_flatmap_1.default(normalizedSidebar, normalizeItem);
        return acc;
    }, {});
}
function loadSidebars(sidebarPaths) {
    // We don't want sidebars to be cached because of hot reloading.
    let allSidebars = {};
    if (!sidebarPaths || !sidebarPaths.length) {
        return {};
    }
    sidebarPaths.map((sidebarPath) => {
        if (sidebarPath && fs_extra_1.default.existsSync(sidebarPath)) {
            const sidebar = import_fresh_1.default(sidebarPath);
            Object.assign(allSidebars, sidebar);
        }
    });
    return normalizeSidebar(allSidebars);
}
exports.default = loadSidebars;
