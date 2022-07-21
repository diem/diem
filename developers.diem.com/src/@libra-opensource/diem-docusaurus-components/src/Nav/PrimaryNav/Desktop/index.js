import React from 'react';

import useBaseUrl from '@docusaurus/useBaseUrl';
import useDocusaurusContext from '@docusaurus/useDocusaurusContext';
import useThemeContext from '@theme/hooks/useThemeContext';

import NavLink, { BUTTON_TYPES } from '../../components/NavLink';
import Logo from '../../../../img/logo.svg';

import classnames from 'classnames';
import styles from './styles.module.css';
import navStyles from '../../styles.module.css';

const PrimaryNavDesktop = () => {
  const { isDarkTheme } = useThemeContext();
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
    <div className={classnames(styles.root, 'desktop-only')}>
      <a className={navStyles.logo} href={logo.to}>
        <Logo alt={logo.alt} />
      </a>
      <ul className={styles.right}>
        {primaryLinks.map(props =>
          <NavLink key={props.label} {...props} />
        )}
        <NavLink
          className={styles['corner-link']}
          type={BUTTON_TYPES.CTA}
          {...cornerLink}
        />
      </ul>
    </div>
  );
}

export default PrimaryNavDesktop
