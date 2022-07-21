import React, {useCallback} from 'react';
import PropTypes from 'prop-types';
import WithBackgroundImage from '../../WithBackgroundImage';

import Toggle from '../../Toggle';
import useThemeContext from '@theme/hooks/useThemeContext';

import classnames from 'classnames';
import styles from './styles.module.css';

const TopBar = ({editUrl}) => {
  const {isDarkTheme, setLightTheme, setDarkTheme} = useThemeContext();

  const onToggleChange = useCallback(
    e => e.target.checked ? setDarkTheme() : setLightTheme(),
    [setLightTheme, setDarkTheme],
  );

  return (
    <div className={styles.root}>
      <WithBackgroundImage
        className={classnames(styles.edit, styles.backgroundIcon)}
        href={editUrl}
        imageLight="/img/shared/edit.svg"
        imageLightHover="/img/shared/edit-hover.svg"
        imageDarkHover="/img/shared/edit-dark-hover.svg"
        tag="a"
        target="_blank"
      >
        Edit
      </WithBackgroundImage>
      <label className={styles.toggle}>
        <Toggle
          className={styles.displayOnlyInLargeViewport}
          aria-label="Dark mode toggle"
          checked={isDarkTheme}
          onChange={onToggleChange}
        />
        <span>
          {isDarkTheme ? "Light" : "Dark"} Mode
        </span>
      </label>
    </div>
  );
};

TopBar.propTypes = {
  editUrl: PropTypes.string.isRequired,
};

export default TopBar;
