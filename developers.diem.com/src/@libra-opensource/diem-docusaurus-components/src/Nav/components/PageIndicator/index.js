import React from 'react';

import useDocusaurusContext from '@docusaurus/useDocusaurusContext';

import styles from './styles.module.css';

const PageIndicator = () => {
  const {
    siteConfig: {
      customFields: {
        navbar: {
          title
        },
      },
    }
  } = useDocusaurusContext();

  return (
    <div className={styles.root}>
      <span className={styles.primary}><b>Developers</b></span>
      <span className={styles.divider}> / </span>
      <span className={styles.secondary}>{title}</span>
    </div>
  );
};

export default PageIndicator;
