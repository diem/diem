import React from 'react';
import PropTypes from 'prop-types';

import useBaseUrl from '@docusaurus/useBaseUrl';
import useDocusaurusContext from '@docusaurus/useDocusaurusContext';

import CloseIcon from '../../../../img/close.svg';
import CornerLink from '../../../../img/white-paper.svg';
import Logo from '../../../../img/logo.svg';
import OpenIcon from '../../../../img/vertical-ellipse.svg';
import PopupMenu from '../../components/PopupMenu';
import NavLink from '../../components/NavLink';
import NavMenuIcon from '../../components/NavMenuIcon';

import classnames from 'classnames';
import navStyles from '../../styles.module.css';
import styles from './styles.module.css';

const PrimaryNavMobile = ({ activePopupMenu, setPopupMenu } ) => {
  const {
    siteConfig: {
      themeConfig: {
        logo,
      },
      customFields: {
        navbar: {
          cornerLink,
          primaryLinks,
        },
      },
    }
  } = useDocusaurusContext();

  return (
    <div className={classnames(styles.root, 'mobile-only')}>
      <div className={styles.mainContainer}>
        <NavMenuIcon
          onClick={() => {setPopupMenu('primary')}}
          CloseIcon={CloseIcon}
          isOpen={activePopupMenu === 'primary'}
          OpenIcon={OpenIcon}
        />

        <a href={logo.href}>
          <Logo alt={logo.alt} className={navStyles.logo} />
        </a>
        <a className={styles.cornerLink} href={cornerLink.to}>
          <CornerLink alt={cornerLink.alt} />
        </a>
      </div>
      {activePopupMenu === 'primary' &&
        <PopupMenu links={primaryLinks} />
      }
    </div>
  );
};

PrimaryNavMobile.propTypes = {
  activePopupMenu: PropTypes.string,
  setPopupMenu: PropTypes.func.isRequired,
};

export default PrimaryNavMobile;
