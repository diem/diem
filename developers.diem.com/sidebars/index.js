const {category, standaloneLink} = require("./components");

const Sidebar = [
  {
    type: 'ref',
    id: 'welcome-to-diem',
    customProps: {
      classNames: ['home'],
      icon: 'img/home.svg',
      iconDark: 'img/home-dark.svg',
    },
  },

  category('Basics', [
    'basics/basics-txns-states',
    'basics/basics-validator-nodes',
    'basics/basics-fullnodes',
    'basics/basics-accounts',
    'basics/basics-gas-txn-fee',
    'basics/basics-events',
    'basics/basics-node-networks-sync',
  ]),

  category('Transactions', [
    'transactions/basics-life-of-txn',
    category('Types of Transactions', [
      'transactions/txns-types',
      'transactions/txns-types/txns-create-accounts-mint',
      'transactions/txns-types/txns-manage-accounts',
      'transactions/txns-types/txns-send-payment',
    ]),
  ]),

  category('Wallets and Merchant Stores', [
    'wallets-and-merchant-stores/integrate-wallet-merchant-dpn',

    category('Diem Reference Wallet', [
      'wallets-and-merchant-stores/diem-reference-wallet',
      'wallets-and-merchant-stores/diem-reference-wallet/reference-wallet-admin-dash',
      'wallets-and-merchant-stores/diem-reference-wallet/reference-wallet-local-mob',
      'wallets-and-merchant-stores/diem-reference-wallet/reference-wallet-local-web',
      'wallets-and-merchant-stores/diem-reference-wallet/reference-wallet-public-demo',
      'wallets-and-merchant-stores/diem-reference-wallet/reference-wallet-set-up-modules',
    ]),

    category('Diem Reference Merchant Store', [
      'wallets-and-merchant-stores/diem-reference-merchant-store',
      'wallets-and-merchant-stores/diem-reference-merchant-store/local-web-reference-merchant',
      'wallets-and-merchant-stores/diem-reference-merchant-store/reference-merchant-manage-payments',
      'wallets-and-merchant-stores/diem-reference-merchant-store/reference-merchant-public-demo',
      'wallets-and-merchant-stores/diem-reference-merchant-store/reference-merchant-set-up-modules',
    ]),

    'wallets-and-merchant-stores/try-our-mini-wallet',
  ]),

  category('Tutorials', [
    'tutorials/tutorial-my-first-transaction',
    'tutorials/tutorial-query-the-blockchain',
    'tutorials/configure-run-public-fullnode',
    'tutorials/tutorial-run-local-validator-nw',
    'tutorials/tutorial-my-first-client',
  ]),

  category('Tools', [
    'tools/sdks',
    standaloneLink('https://github.com/diem/diem/blob/main/json-rpc/json-rpc-spec.md', 'JSON-RPC API'),
    'tools/cli-reference',
    'tools/github-projects',
  ]),

  category('Reference', [
    'reference/reference-rust-docs',
    'reference/security',
    'reference/glossary',
  ]),

  category('Technical Papers', [
    'technical-papers/technical-papers-overview',
    'technical-papers/move-paper',
    'technical-papers/the-diem-blockchain-paper',
    'technical-papers/state-machine-replication-paper',
    'technical-papers/jellyfish-merkle-tree-paper',
    'technical-papers/publication-archive',
  ]),

  category('Policies', [
    'policies/terms-of-use',
    'policies/code-of-conduct',
    'policies/cookies',
    'policies/coding-guidelines',
    'policies/contributing',
    'policies/privacy-policy',
    'policies/maintainers',
  ]),

];


module.exports = {Sidebar};
