/**
 * Copyright (c) The Libra Core Contributors
 * SPDX-License-Identifier: Apache-2.0
 */

const React = require('react');

const CompLibrary = require('../../core/CompLibrary.js');

const Container = CompLibrary.Container;
const GridBlock = CompLibrary.GridBlock;
const papersLocation = '/docs/assets/papers';

// Metadata on the paper versions. Whenever a new paper is added just add the
// new date at the top of the corresponding "dates" array.
// TODO (joshua): Automate the list of papers by looking at the directories.
const paperMeta = {
  'The Libra Blockchain': {
    abstractUrl: '/docs/the-libra-blockchain-paper/',
    paperBase: `${papersLocation}/the-libra-blockchain`,
    dates: ['2020-05-26', '2020-04-09', '2019-09-26', '2019-09-18', '2019-06-25'],
    imgLoc: '/docs/assets/illustrations/libra-blockchain-pdf.png',
    imgAlt: 'The Libra Blockchain PDF Download',
  },
  'Move Programming Language': {
    abstractUrl: '/docs/move-paper/',
    paperBase: `${papersLocation}/libra-move-a-language-with-programmable-resources`,
    dates: ['2020-05-26', '2020-04-09', '2019-09-26', '2019-06-18'],
    imgLoc: '/docs/assets/illustrations/move-language-pdf.png',
    imgAlt: 'Move: A Language With Programmable Resources PDF Download',
  },
  'State Machine Replication': {
    abstractUrl: '/docs/state-machine-replication-paper/',
    paperBase: `${papersLocation}/libra-consensus-state-machine-replication-in-the-libra-blockchain`,
    dates: [
      '2020-05-26',
      '2020-04-09',
      '2019-11-08',
      '2019-10-24',
      '2019-09-26',
      '2019-09-19',
      '2019-06-28',
    ],
    imgLoc: '/docs/assets/illustrations/state-machine-pdf.png',
    imgAlt: 'State Machine Replication in the Libra Blockchain PDF Download',
  },
};

/**
 * Get the sections for each of the papers in the paperMetadata.
 */
function getPapers() {
  const paperSections = [];

  for (const paper in paperMeta) {
    const metadata = paperMeta[paper];

    const dates = Array.from(metadata.dates);
    const current = dates.shift();
    const oldPapers = dates.map(function (date, idx) {
      return (
        <li key={`${metadata.paperBase}--${idx}`}>
          <a href={`${metadata.paperBase}/${date}.pdf`}>{date}</a>
        </li>
      );
    });

    paperSections.push(
      <section key={metadata.paperBase}>
        <h2>
          <a href={metadata.abstractUrl}>{paper}</a>
        </h2>
        <ul>
          <li>
            <a href={`${metadata.paperBase}/${current}.pdf`}>{`Latest version (${current})`}</a>
          </li>
          {oldPapers}
        </ul>
      </section>,
    );
  }

  return paperSections;
}

function Papers(props) {
  return (
    <Container>
      <h1>Publication Archive</h1>
      {getPapers()}
    </Container>
  );
}

module.exports = Papers;
