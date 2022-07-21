import React from 'react';
import PropTypes from 'prop-types';

import Desktop from './Desktop';
import Mobile from './Mobile';

import styles from './styles.module.css';

const PrimaryNav = ({ activePopupMenu, setPopupMenu }) => (
  <div className={styles.root}>
    <div className="width-wrapper diem-org-width">
      <Mobile activePopupMenu={activePopupMenu} setPopupMenu={setPopupMenu} />
      <Desktop />
    </div>
  </div>
);

PrimaryNav.propTypes = {
  activePopupMenu: PropTypes.string,
  setPopupMenu: PropTypes.func.isRequired,
};

export default PrimaryNav;
