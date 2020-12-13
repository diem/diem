import React from 'react';
import Terminal from 'diem-cli';

import './styles.css';
import classnames from 'classnames';
import styles from './styles.module.css';

const CLI = () => {
  return <Terminal className={classnames("cli", styles.root)} />;
};

export default CLI;
