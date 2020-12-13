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
const lodash_groupby_1 = __importDefault(require("lodash.groupby"));
const lodash_pick_1 = __importDefault(require("lodash.pick"));
const lodash_pickby_1 = __importDefault(require("lodash.pickby"));
const globby_1 = __importDefault(require("globby"));
const fs_extra_1 = __importDefault(require("fs-extra"));
const path_1 = __importDefault(require("path"));
const remark_admonitions_1 = __importDefault(require("remark-admonitions"));
const utils_1 = require("@docusaurus/utils");
const order_1 = __importDefault(require("./order"));
const sidebars_1 = __importDefault(require("./sidebars"));
const metadata_1 = __importDefault(require("./metadata"));
const env_1 = __importDefault(require("./env"));
const version_1 = require("./version");
const DEFAULT_OPTIONS = {
    path: 'docs',
    routeBasePath: 'docs',
    include: ['**/*.{md,mdx}'],
    sidebarPath: '',
    docLayoutComponent: '@theme/DocPage',
    docItemComponent: '@theme/DocItem',
    remarkPlugins: [],
    rehypePlugins: [],
    showLastUpdateTime: false,
    showLastUpdateAuthor: false,
    admonitions: {},
};
function pluginContentDocs(context, opts) {
    const options = Object.assign(Object.assign({}, DEFAULT_OPTIONS), opts);
    if (options.admonitions) {
        options.remarkPlugins = options.remarkPlugins.concat([
            [remark_admonitions_1.default, options.admonitions],
        ]);
    }
    const { siteDir, generatedFilesDir, baseUrl } = context;
    const docsDir = path_1.default.resolve(siteDir, options.path);
    const sourceToPermalink = {};
    const dataDir = path_1.default.join(generatedFilesDir, 'docusaurus-plugin-content-docs');
    // Versioning.
    const env = env_1.default(siteDir);
    const { versioning } = env;
    const { versions, docsDir: versionedDir, sidebarsDir: versionedSidebarsDir, } = versioning;
    const versionsNames = versions.map((version) => `version-${version}`);
    return {
        name: 'docusaurus-plugin-content-docs',
        extendCli(cli) {
            cli
                .command('docs:version')
                .arguments('<version>')
                .description('Tag a new version for docs')
                .action((version) => {
                version_1.docsVersion(version, siteDir, {
                    path: options.path,
                    sidebarPath: options.sidebarPath,
                });
            });
        },
        getPathsToWatch() {
            const { include } = options;
            let globPattern = include.map((pattern) => `${docsDir}/${pattern}`);
            if (versioning.enabled) {
                const docsGlob = include
                    .map((pattern) => versionsNames.map((versionName) => `${versionedDir}/${versionName}/${pattern}`))
                    .reduce((a, b) => a.concat(b), []);
                const sidebarsGlob = versionsNames.map((versionName) => `${versionedSidebarsDir}/${versionName}-sidebars.json`);
                globPattern = [...globPattern, ...sidebarsGlob, ...docsGlob];
            }
            return [...globPattern, options.sidebarPath];
        },
        getClientModules() {
            const modules = [];
            if (options.admonitions) {
                modules.push('remark-admonitions/styles/infima.css');
            }
            return modules;
        },
        // Fetches blog contents and returns metadata for the contents.
        async loadContent() {
            const { include, sidebarPath } = options;
            if (!fs_extra_1.default.existsSync(docsDir)) {
                return null;
            }
            // Prepare metadata container.
            const docsMetadataRaw = {};
            const docsPromises = [];
            // Metadata for default/master docs files.
            const docsFiles = await globby_1.default(include, {
                cwd: docsDir,
            });
            docsPromises.push(Promise.all(docsFiles.map(async (source) => {
                const metadata = await metadata_1.default({
                    source,
                    refDir: docsDir,
                    context,
                    options,
                    env,
                });
                docsMetadataRaw[metadata.id] = metadata;
            })));
            // Metadata for versioned docs.
            if (versioning.enabled) {
                const versionedGlob = include
                    .map((pattern) => versionsNames.map((versionName) => `${versionName}/${pattern}`))
                    .reduce((a, b) => a.concat(b), []);
                const versionedFiles = await globby_1.default(versionedGlob, {
                    cwd: versionedDir,
                });
                docsPromises.push(Promise.all(versionedFiles.map(async (source) => {
                    const metadata = await metadata_1.default({
                        source,
                        refDir: versionedDir,
                        context,
                        options,
                        env,
                    });
                    docsMetadataRaw[metadata.id] = metadata;
                })));
            }
            // Load the sidebars and create docs ordering.
            const sidebarPaths = [
                sidebarPath,
                ...versionsNames.map((versionName) => `${versionedSidebarsDir}/${versionName}-sidebars.json`),
            ];
            const loadedSidebars = sidebars_1.default(sidebarPaths);
            const order = order_1.default(loadedSidebars);
            await Promise.all(docsPromises);
            // Construct inter-metadata relationship in docsMetadata.
            const docsMetadata = {};
            const permalinkToSidebar = {};
            const versionToSidebars = {};
            Object.keys(docsMetadataRaw).forEach((currentID) => {
                var _a, _b, _c, _d, _e, _f;
                const { next: nextID, previous: previousID, sidebar } = order[currentID] || {};
                const previous = previousID
                    ? {
                        title: (_b = (_a = docsMetadataRaw[previousID]) === null || _a === void 0 ? void 0 : _a.title) !== null && _b !== void 0 ? _b : 'Previous',
                        permalink: (_c = docsMetadataRaw[previousID]) === null || _c === void 0 ? void 0 : _c.permalink,
                    }
                    : undefined;
                const next = nextID
                    ? {
                        title: (_e = (_d = docsMetadataRaw[nextID]) === null || _d === void 0 ? void 0 : _d.title) !== null && _e !== void 0 ? _e : 'Next',
                        permalink: (_f = docsMetadataRaw[nextID]) === null || _f === void 0 ? void 0 : _f.permalink,
                    }
                    : undefined;
                docsMetadata[currentID] = Object.assign(Object.assign({}, docsMetadataRaw[currentID]), { sidebar,
                    previous,
                    next });
                // sourceToPermalink and permalinkToSidebar mapping.
                const { source, permalink, version } = docsMetadataRaw[currentID];
                sourceToPermalink[source] = permalink;
                if (sidebar) {
                    permalinkToSidebar[permalink] = sidebar;
                    if (versioning.enabled && version) {
                        if (!versionToSidebars[version]) {
                            versionToSidebars[version] = new Set();
                        }
                        versionToSidebars[version].add(sidebar);
                    }
                }
            });
            const convertDocLink = (item) => {
                const linkID = item.id;
                const linkMetadata = docsMetadataRaw[linkID];
                if (!linkMetadata) {
                    throw new Error(`Improper sidebars file, document with id '${linkID}' not found.`);
                }
                return {
                    type: 'link',
                    label: linkMetadata.sidebar_label || linkMetadata.title,
                    href: linkMetadata.permalink,
                    extra: item.extra,
                };
            };
            const normalizeItem = (item) => {
                switch (item.type) {
                    case 'category':
                        return Object.assign(Object.assign({}, item), { items: item.items.map(normalizeItem) });
                    case 'ref':
                    case 'doc':
                        return convertDocLink(item);
                    case 'link':
                    default:
                        return item;
                }
            };
            // Transform the sidebar so that all sidebar item will be in the
            // form of 'link' or 'category' only.
            // This is what will be passed as props to the UI component.
            const docsSidebars = Object.entries(loadedSidebars).reduce((acc, [sidebarId, sidebarItems]) => {
                acc[sidebarId] = sidebarItems.map(normalizeItem);
                return acc;
            }, {});
            return {
                docsMetadata,
                docsDir,
                docsSidebars,
                permalinkToSidebar: utils_1.objectWithKeySorted(permalinkToSidebar),
                versionToSidebars,
            };
        },
        async contentLoaded({ content, actions }) {
            if (!content || Object.keys(content.docsMetadata).length === 0) {
                return;
            }
            const { docLayoutComponent, docItemComponent, routeBasePath } = options;
            const { addRoute, createData } = actions;
            const aliasedSource = (source) => `~docs/${path_1.default.relative(dataDir, source)}`;
            const genRoutes = async (metadataItems) => {
                const routes = await Promise.all(metadataItems.map(async (metadataItem) => {
                    await createData(
                    // Note that this created data path must be in sync with
                    // metadataPath provided to mdx-loader.
                    `${utils_1.docuHash(metadataItem.source)}.json`, JSON.stringify(metadataItem, null, 2));
                    return {
                        path: metadataItem.permalink,
                        component: docItemComponent,
                        exact: true,
                        modules: {
                            content: metadataItem.source,
                        },
                    };
                }));
                return routes.sort((a, b) => a.path > b.path ? 1 : b.path > a.path ? -1 : 0);
            };
            const addBaseRoute = async (docsBaseRoute, docsBaseMetadata, routes, priority) => {
                const docsBaseMetadataPath = await createData(`${utils_1.docuHash(docsBaseRoute)}.json`, JSON.stringify(docsBaseMetadata, null, 2));
                addRoute({
                    path: docsBaseRoute,
                    component: docLayoutComponent,
                    routes,
                    modules: {
                        docsMetadata: aliasedSource(docsBaseMetadataPath),
                    },
                    priority,
                });
            };
            // If versioning is enabled, we cleverly chunk the generated routes
            // to be by version and pick only needed base metadata.
            if (versioning.enabled) {
                const docsMetadataByVersion = lodash_groupby_1.default(Object.values(content.docsMetadata), 'version');
                await Promise.all(Object.keys(docsMetadataByVersion).map(async (version) => {
                    const routes = await genRoutes(docsMetadataByVersion[version]);
                    const isLatestVersion = version === versioning.latestVersion;
                    const docsBasePermalink = utils_1.normalizeUrl([
                        baseUrl,
                        routeBasePath,
                        isLatestVersion ? '' : version,
                    ]);
                    const docsBaseRoute = utils_1.normalizeUrl([docsBasePermalink, ':route']);
                    const neededSidebars = content.versionToSidebars[version] || new Set();
                    const docsBaseMetadata = {
                        docsSidebars: lodash_pick_1.default(content.docsSidebars, Array.from(neededSidebars)),
                        permalinkToSidebar: lodash_pickby_1.default(content.permalinkToSidebar, (sidebar) => neededSidebars.has(sidebar)),
                        version,
                    };
                    // We want latest version route config to be placed last in the
                    // generated routeconfig. Otherwise, `/docs/next/foo` will match
                    // `/docs/:route` instead of `/docs/next/:route`.
                    return addBaseRoute(docsBaseRoute, docsBaseMetadata, routes, isLatestVersion ? -1 : undefined);
                }));
            }
            else {
                const routes = await genRoutes(Object.values(content.docsMetadata));
                const docsBaseMetadata = {
                    docsSidebars: content.docsSidebars,
                    permalinkToSidebar: content.permalinkToSidebar,
                };
                const docsBaseRoute = utils_1.normalizeUrl([baseUrl, routeBasePath, ':route']);
                return addBaseRoute(docsBaseRoute, docsBaseMetadata, routes);
            }
        },
        configureWebpack(_config, isServer, utils) {
            const { getBabelLoader, getCacheLoader } = utils;
            const { rehypePlugins, remarkPlugins } = options;
            return {
                resolve: {
                    alias: {
                        '~docs': dataDir,
                    },
                },
                module: {
                    rules: [
                        {
                            test: /(\.mdx?)$/,
                            include: [docsDir, versionedDir].filter(Boolean),
                            use: [
                                getCacheLoader(isServer),
                                getBabelLoader(isServer),
                                {
                                    loader: '@docusaurus/mdx-loader',
                                    options: {
                                        remarkPlugins,
                                        rehypePlugins,
                                        metadataPath: (mdxPath) => {
                                            // Note that metadataPath must be the same/in-sync as
                                            // the path from createData for each MDX.
                                            const aliasedSource = utils_1.aliasedSitePath(mdxPath, siteDir);
                                            return path_1.default.join(dataDir, `${utils_1.docuHash(aliasedSource)}.json`);
                                        },
                                    },
                                },
                                {
                                    loader: path_1.default.resolve(__dirname, './markdown/index.js'),
                                    options: {
                                        siteDir,
                                        docsDir,
                                        sourceToPermalink: sourceToPermalink,
                                        versionedDir,
                                    },
                                },
                            ].filter(Boolean),
                        },
                    ],
                },
            };
        },
    };
}
exports.default = pluginContentDocs;
