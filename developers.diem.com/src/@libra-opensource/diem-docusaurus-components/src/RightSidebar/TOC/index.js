import React from 'react';
import PropTypes from 'prop-types';

import useTOCHighlight from '@theme/hooks/useTOCHighlight';

import styles from './styles.module.css';

const LINK_CLASS_NAME = styles['contentsLink'];
const ACTIVE_LINK_CLASS_NAME = 'contents__link--active';
const TOP_OFFSET = 140;

function Headings({ headings, isChild }) {
  if (!headings.length) {
    return null;
  }
  return (
    <ul className={styles.category}>
      {headings.map((heading) => (
        <li key={heading.id} className={styles.heading}>
          <a
            href={`#${heading.id}`}
            className={LINK_CLASS_NAME}
            dangerouslySetInnerHTML={{__html: heading.value}}
          />
          <Headings isChild headings={heading.children} />
        </li>
      ))}
    </ul>
  );
}

const TOC = ({ headings }) => {
  useTOCHighlight(LINK_CLASS_NAME, ACTIVE_LINK_CLASS_NAME, TOP_OFFSET);
  return (
    <div className={styles.root}>
      <span className={styles.title}>On This Page</span>
      <Headings headings={headings} />
    </div>
  );
}

TOC.propTypes = {
  headings: PropTypes.array.isRequired,
};

TOC.defaultProps = {
  headings: [],
};

export default TOC;
