import React from 'react';
import PropTypes from 'prop-types';

import styles from './styles.module.css';
import classnames from 'classnames';

const BlockQuote = ({children, type = "info"}) => (
  <blockquote className={classnames(styles.blockquote, styles[type])}>
    {children}
  </blockquote>
);

BlockQuote.propTypes = {
  children: PropTypes.oneOfType([PropTypes.array, PropTypes.string]).isRequired,
  type: PropTypes.oneOf(["info", "warning", "danger"]),
};

export default BlockQuote;
