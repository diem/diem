/**
 * Copyright (c) The Libra Core Contributors
 * SPDX-License-Identifier: CC-BY-4.0
 *
 * @format
 */

// See https://docusaurus.io/docs/site-config for all the possible
// site configuration options.

const markdownPlugins = require(`${process.cwd()}/markdownPlugins.js`);

// Define this so it can be easily modified in scripts (to host elsewhere)
const baseUrl = '/';

// List of projects/orgs using your project for the users page.
const users = [];

const siteConfig = {
  title: 'Libra',
  tagline:
    'The Libra Association’s mission is to enable a simple global payment system and financial infrastructure that empowers billions of people.',
  url: 'https://developers.libra.org',
  baseUrl: baseUrl,
  cleanUrl: true, // No .html extensions for paths
  blogSidebarCount: 'ALL', // Show all blog posts
  headerIcon: 'img/libra-nav-logo.svg',
  headerLogoUrl: 'https://libra.org',
  footerIcon: 'img/libra-logomark-white.png',
  favicon: 'img/libra.ico',
  cname: 'developers.libra.org',

  // used for publishing and more
  organizationName: 'libra',
  projectName: 'libra',

  // links that will be used in the header navigation bar
  headerLinks: [
    { href: 'https://libra.org/vision/', label: 'Vision' },
    { href: 'https://libra.org/association/', label: 'Association' },
    {
      href: 'https://libra.org/developers/',
      label: 'Developers',
      selected: true,
    },
    { href: 'https://libra.org/learn-faqs/', label: 'Learn' },
    { href: 'https://libra.org/media-press-news/', label: 'Media' },
    {
      href: 'https://libra.org/white-paper/',
      label: 'White Paper',
      highlight: true,
      mobileImg: {
        image: '/img/white-paper.svg',
        alt: 'White Paper',
      },
      mobileMain: true,
    },
    // {search: false}, // position search box to the very right
  ],

  subHeaderLinks: [
    { href: 'https://libra.org/developers/', label: 'Overview' },
    { doc: 'welcome-to-libra', label: 'Libra Docs' },
    { href: '/docs/rustdocs/', label: 'Libra Rust Crates' },
    { href: '/docs/python-client-sdk-docs/libra/', label: 'Libra Python Client SDK' },
    { href: 'https://lip.libra.org', label: 'Governance' },
    { href: 'https://community.libra.org', label: 'Community' },
    { href: 'https://github.com/libra/libra', label: 'GitHub', external: true },
    { search: true },
  ],

  // add users to the website
  users,

  // search integration w/ algolia

  // This website manually inserts the Algolia Search bar in Footer.js
  algolia: {
    apiKey: '0d48ee629d39ddc4916eeef7755a0c4c',
    indexName: 'libra',
  },

  // colors for website
  colors: {
    primaryColor: '#3333ff', // dark blue
    secondaryColor: '#aaaaff', // light blue
  },

  highlight: {
    theme: 'default',
  },

  // custom scripts that are placed in <head></head> of each page
  scripts: [
    // Github buttons
    `${baseUrl}js/buttons.js`,
    // Copy-to-clipboard button for code blocks
    `${baseUrl}js/code_block_buttons.js`,
    // Manages the cookie banner
    `${baseUrl}js/cookie_banner.js`,
    // Manages disable ad blocker modal
    `${baseUrl}js/disable_adblock.js`,
    // From https://cdnjs.cloudflare.com/ajax/libs/clipboard.js/2.0.0/clipboard.min.js
    // Segment analytics for the form data. Make sure to load the analytics 1st
    `${baseUrl}js/segment.analytics.min.js`,
    `${baseUrl}js/segment.js`,
    `${baseUrl}js/clipboardjs.2.0.0.min.js`,
    `${baseUrl}js/forms.js`,
    `${baseUrl}js/mobile_nav.js`,
    // `${baseUrl}js/docsearch.min.js`,
    // `${baseUrl}js/search.js`,
  ],

  // Custom markdown functions
  markdownPlugins: markdownPlugins,

  // enable on-page navigation for the current documentation page
  onPageNav: 'separate',

  // enable scroll to top button a the bottom of the site
  // scrollToTop: true,

  // if true, expand/collapse links & subcategories in sidebar
  docsSideNavCollapsible: false,

  // URL for editing docs
  editUrl: 'https://github.com/libra/libra/edit/master/developers.libra.org/docs/',

  // Open Graph and Twitter card images
  ogImage: 'img/libra.png',
  twitterImage: 'img/libra.png',

  // custom highlighter for Move
  highlight: {
    // The name of the theme used by Highlight.js when highlighting code.
    theme: 'default',

    // Default language.
    defaultLang: 'plaintext',

    // Highlighting for Move.
    // NB: This is not correct for the whole Move grammar but just for
    // the examples on the site!
    hljs: function (hljs) {
      hljs.registerLanguage('move', function (hljs) {
        var KEYWORDS = [
          'public',
          'module',
          'import',
          'else',
          'if',
          'let',
          'return',
          'copy',
          'move',
          'struct',
          'resource',
          'mut',
        ].join(' ');
        var BUILTINS = [
          'bytearray',
          'get_txn_sender',
          'move_from',
          'create_account',
          'bool',
          'address',
          'u64',
          'move_to_sender',
          'assert',
        ].join(' ');
        var LITERALS = ['true', 'false'].join(' ');
        var TYPES = {
          className: 'type',
          begin: /[A-Z][a-zA-Z0-9_#]*/,
        };
        var NUMBERS = {
          className: 'number',
          variants: [{ begin: '\\b0x([A-Fa-f0-9_]+)' }, { begin: '\\b(\\d+)' }],
        };
        var STRUCTS = {
          className: 'struct',
          beginKeywords: 'struct resource',
          end: '{',
        };
        return {
          keywords: {
            keyword: KEYWORDS,
            literal: LITERALS,
            built_in: BUILTINS,
          },
          contains: [TYPES, NUMBERS, STRUCTS, hljs.C_LINE_COMMENT_MODE],
        };
      });
    },
  },

  // show html docs generated by rustdoc
  separateCss: ['static/docs/rustdocs'],
};

module.exports = siteConfig;
