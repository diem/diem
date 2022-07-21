import React from 'react';
import PropTypes from 'prop-types';

import BaseContainer from '../BaseContainer';
import Link from 'src/components/Link';
import {WithBackgroundImage} from 'diem-docusaurus-components';

import classnames from 'classnames';
import styles from './styles.module.css';

const SDKCard = ({bolded, docs, icon, iconDark, overlay, title, sdk, smallerImage, to}) => {
  return (
    <BaseContainer className={styles.root} to={to} overlay={overlay}>
      <a href={sdk} className={styles.left} title={`${title} SDK`}>
        <WithBackgroundImage
          className={styles.icon}
          imageLight={icon}
          imageDark={iconDark}
        />
        <div className={classnames(styles.label, styles.underIconText)}>{title}</div>
      </a>

      <div className={styles.right}>
        <Link className={styles.sdk} href={sdk} title={`${title} SDK`}>
          <WithBackgroundImage
            className={styles.buttonImage}
            imageLight="img/document.svg"
            imageDark="img/document-dark.svg"
          />
          <span className={styles.label}>SDK</span>
        </Link>
        {docs &&
        <Link className={styles.docs} href={docs} title={`${title} docs`}>
          <WithBackgroundImage
            className={styles.buttonImage}
            imageLight="img/roadmap.png"
            imageDark="img/reference-dark.svg"
          />
          <span className={styles.label}>Docs</span>
        </Link>
        }
      </div>
    </BaseContainer>
  )
}

SDKCard.propTypes = {
  docs: PropTypes.string,
  icon: PropTypes.string.isRequired,
  iconDark: PropTypes.string,
  overlay: PropTypes.string,
  title: PropTypes.string,
  sdk: PropTypes.string.isRequired,
  to: PropTypes.string,
};

export default SDKCard;
