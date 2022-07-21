import React from 'react';
import PropTypes from 'prop-types';

function versionToRow(version, i) {
  let [date, url] = version;
  const text = i === 0 ? `Latest version (${date})` : date;
  return (
    <li key={date}>
      <a href={url}>{text}</a>
    </li>
  )
}

const PublicationArchiveList = ({title, doc_link, versions}) => (
  <section>
    <h2><a href={doc_link}>{title}</a></h2>
    <ul>
      {versions.map(versionToRow)}
    </ul>
  </section>
)

PublicationArchiveList.propTypes = {
  title: PropTypes.string.isRequired,
  doc_link: PropTypes.string.isRequired,
  // [ [date, url], [date, url], [date, url], ... ]
  versions: PropTypes.arrayOf(PropTypes.arrayOf(PropTypes.string)),

};

export default PublicationArchiveList;
