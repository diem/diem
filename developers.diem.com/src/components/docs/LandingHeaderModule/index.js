import React from 'react';
import PropTypes from 'prop-types';
import Link from "@docusaurus/Link";

import styles from './styles.module.css';

const LandingHeaderModule = ({copy, cta, ctaLink, img, imgAlt, noArrow, title}) => (
  <div className={styles.root}>
    <div className={styles.content}>
      <h1 className={styles.title}>{title}</h1>
      <p>{copy}</p>

      <Link className={styles.cta} href={ctaLink}>
        <span className={styles.join}>
          <div className="buttonWrapper">
            <span className={styles.button + " button"}>
          {cta}
            </span>
          </div>
        </span>
      </Link>

    </div>
    <img alt={imgAlt || "Marketing Module Image"} src={img}/>
  </div>
);


LandingHeaderModule.propTypes = {
  copy: PropTypes.string.isRequired,
  cta: PropTypes.string.isRequired,
  ctaLink: PropTypes.string.isRequired,
  img: PropTypes.string.isRequired,
  imgAlt: PropTypes.string,
  noArrow: PropTypes.bool,
  title: PropTypes.string,
};

export default LandingHeaderModule;
