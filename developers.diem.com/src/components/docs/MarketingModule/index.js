import React from 'react';
import PropTypes from 'prop-types';
import Link from "@docusaurus/Link";

import styles from './styles.module.css';
import Arrow from 'img/marketing-arrow.svg';

const MarketingModule = ({copy, cta, ctaLink, img, imgAlt, title}) => (
  <Link className={styles.cta} href={ctaLink}>

    <div className={styles.root}>

      <div className={styles.grid}>

        <div className={styles.leftTop}>
          <div className={styles.ctaTop}>{cta}</div>
          <h2 className={styles.title}>{title}</h2>
          <p className={styles.copy}>{copy}</p>
        </div>

        <div className={styles.leftBottom}>
          <div className={styles.arrowHolder}>
            <span className={styles.arrow}>
            <Arrow/>
          </span>
          </div>
        </div>

      </div>

      <div className={styles.imgHolder}>
        <img src={img} alt={imgAlt}/>
      </div>

    </div>
  </Link>

);

MarketingModule.propTypes = {
  copy: PropTypes.string.isRequired,
  cta: PropTypes.string,
  ctaLink: PropTypes.string,
  img: PropTypes.string.isRequired,
  imgAlt: PropTypes.string,
  title: PropTypes.string,
};

export default MarketingModule;
