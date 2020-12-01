const base = 'https://developers.diem.com';

let path = window.location.pathname.split('#')[0].replace(/\/$/, '');

const mapping = new Map([
  ['/docs/community/coding-guidelines', '/docs/core/coding-guidelines/'],
  ['/docs/community/contributing', '/docs/core/contributing/'],
  ['/docs/libra-core-overview', '/docs/core/overview/'],
  ['/docs/libra-open-source-paper', '/docs/welcome-to-diem/'],
  ['/docs/libra-protocol', '/docs/core/diem-protocol/'],
  ['/docs/life-of-a-transaction', '/docs/core/life-of-a-transaction/'],
  ['/docs/move-overview', '/docs/move/overview/'],
  ['/docs/move-paper', '/docs/technical-papers/move-paper/'],
  ['/docs/my-first-transaction', '/docs/core/my-first-transaction/'],
  ['/docs/policies/security', '/docs/reference/security/'],
  ['/docs/reference/libra-cli', '/docs/cli/'],
  ['/docs/run-local-network', '/docs/core/run-local-network/'],
  ['/docs/run-move-locally', '/docs/move/overview/'],
  ['/docs/state-machine-replication-paper', '/docs/technical-papers/state-machine-replication-paper/'],
  ['/docs/move-paper', '/docs/technical-papers/move-paper/'],
  ['/docs/the-libra-blockchain-paper', '/docs/technical-papers/the-diem-blockchain-paper/'],
  ['/docs/welcome-to-libra', '/docs/welcome-to-diem/'],
]);

const hardcodedRedirect = mapping.get(path);

if (hardcodedRedirect) {
  window.location.href = `${base}${hardcodedRedirect}`;
} else if (
  path.includes('rustdocs') ||
  path.includes('python-client-sdk-docs') ||
  path.includes('crates')) {
  window.location.href = base;
} else {
  window.location.href = `${base}${path}`;
}
