import React, {Component, useState} from 'react';

import PrimaryNav from './PrimaryNav';
import SubNav from './SubNav';

import Variables from '../variables.module.css'
import styles from './styles.module.css';

const Navbar = () => {
  const [activePopupMenu, setActivePopupMenu] = useState(null);

  const setPopupMenu = activePopupMenu => {
    setActivePopupMenu(activePopupMenu);

    if (activePopupMenu !== null) {
      document.querySelector('body').addEventListener('click', function() {
        setActivePopupMenu(null);
      }, { once: true });
    }
  };

  return (
    <nav aria-label="Diem cross-domain nav" className={styles.root}>
      <PrimaryNav activePopupMenu={activePopupMenu} setPopupMenu={setPopupMenu} />
      <SubNav activePopupMenu={activePopupMenu} setPopupMenu={setPopupMenu} />
    </nav>
  );
};

export default Navbar;
