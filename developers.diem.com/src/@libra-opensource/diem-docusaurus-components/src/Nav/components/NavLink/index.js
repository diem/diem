import React from 'react';
import PropTypes from 'prop-types';

import isInternalUrl from '@docusaurus/isInternalUrl';
import useBaseUrl from '@docusaurus/useBaseUrl';
import useThemeContext from '@theme/hooks/useThemeContext';
import useDocusaurusContext from '@docusaurus/useDocusaurusContext';

import classNames from 'classnames';
import styles from './styles.module.css';

export const BUTTON_TYPES = {
  DEFAULT: 'default',
  CTA: 'cta',
}

const NavLink = ({ id, isExternal, label, to, type }) => {
  const {siteConfig: {themeConfig}} = useDocusaurusContext();
  const {siteID} = themeConfig;

  const href = isInternalUrl(to) ? useBaseUrl(to) : to;

  return (
    <li className={classNames(
      styles.root,
      styles[type],
      {active: isInternalUrl(href) || siteID === id},
    )}>
      <a
        href={href}
        target={isExternal ? "_blank" : "_self"}
      >
        {label}
      </a>
    </li>
  );
}

NavLink.propTypes = {
  isExternal: PropTypes.bool,
  label: PropTypes.string.isRequired,
  type: PropTypes.oneOf(Object.values(BUTTON_TYPES)),
  to: PropTypes.string.isRequired,
}

NavLink.defaultProps = {
  isExternal: false,
  type: BUTTON_TYPES.DEFAULT,
}

export default NavLink;
