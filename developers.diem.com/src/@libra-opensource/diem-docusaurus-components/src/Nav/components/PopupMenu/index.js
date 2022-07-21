import React, {useEffect, createRef} from 'react';
import PropTypes from 'prop-types';

import NavLink from '../NavLink';

import styles from './styles.module.css';

const PopupMenu = ({children, links, onClick}) => {
  let containerRef;
  useEffect(() => {
    const onClick = e => {
      e.stopPropagation();
    };

    containerRef.addEventListener('click', onClick);
  });

  return (
    <div className={styles.root} ref={el => containerRef = el}>
      <div className={styles.menu}>
        {links.map(props =>
          <NavLink key={props.label} {...props} />
        )}
        {children}
      </div>
    </div>
  );
};

PopupMenu.propTypes = {
  links: PropTypes.array.isRequired,
};

export default PopupMenu;
