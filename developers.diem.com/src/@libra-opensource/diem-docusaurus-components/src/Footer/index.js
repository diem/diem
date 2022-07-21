import React from "react";

import Link from "@docusaurus/Link";
import useDocusaurusContext from "@docusaurus/useDocusaurusContext";
import useBaseUrl from "@docusaurus/useBaseUrl";

import Logo from "img/shared/logo.svg";
import SocialLinks from "./SocialLinks";

import WithBackgroundImage from "../WithBackgroundImage";

import classnames from "classnames";
import styles from "./styles.module.css";

const Footer = () => {
  const {
    siteConfig: {
      themeConfig: { footer, logo },
      customFields: { socialLinks },
    },
  } = useDocusaurusContext();

  const { copyright, links = [] } = footer;

  return (
    <footer>
      <div className={styles.spacer}>
        <div className={styles.container}>
          <div className={styles.logo}>
            <a href={logo.to}>
              <Logo alt={logo.alt} />
            </a>
          </div>
          {links.map(({ items }, i) => (
            <ul className={classnames(styles.linkList)} key={i}>
              {items.map(({ label, to, type }) => (
                <li className={styles[type]} key={`${label}${to}`}>
                  <a href={to}>{label}</a>
                </li>
              ))}
            </ul>
          ))}
          <div className={styles.connect}>
            <SocialLinks links={socialLinks} />
            <WithBackgroundImage
              className={styles.newsletter}
              href="https://developers.diem.com/newsletter_form"
              imageLight="/img/shared/newsletter.svg"
              imageLightHover="/img/shared/newsletter-hover.svg"
              imageDark="img/shared/newsletter-dark.svg"
              imageDarkHover="img/shared/newsletter-dark-hover.svg"
              tag="a"
              target="_blank"
              type="button"
            >
              Join the Newsletter
            </WithBackgroundImage>
          </div>
        </div>
        <div className={styles.copyright}>{copyright}</div>
      </div>
    </footer>
  );
};

export default Footer;
