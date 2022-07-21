import React from 'react';
import PropTypes from 'prop-types'

import styles from './styles.module.css';
import classnames from 'classnames';

const CardsWrapper = ({cardsPerRow, children, title, justify = false}) => {

  let classes = classnames(styles.root, styles[`rowOf${cardsPerRow}`], {
    [styles.justify]: justify,
  });
  return (
    <>
      {title && <h2>{title}</h2>}
      <div className={classes}>
        {children}
      </div>
    </>
  );
};

CardsWrapper.propTypes = {
  cardsPerRow: PropTypes.number,
  title: PropTypes.string,
  justify: PropTypes.bool,
};

CardsWrapper.defaultProps = {
  cardsPerRow: 3,
};

export default CardsWrapper;
