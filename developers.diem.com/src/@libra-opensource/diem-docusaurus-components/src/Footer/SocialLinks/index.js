import React from 'react';

import Facebook from '../img/facebook.svg';
import LinkedIn from '../img/linked-in.svg';
import Twitter from '../img/twitter.svg';
import Instagram from '../img/instagram.svg';
import Github from '../img/github.svg';

import styles from './styles.module.css';

const Link = ({to, icon}) => (
  <li>
    <a href={to} target="_blank" rel="noopener noreferrer">
      {icon}
    </a>
  </li>
);

const SocialLinks = ({
  links: {
    facebook,
    linkedIn,
    twitter,
    instagram,
    github,
  }
}) => {
  return (
    <ul className={styles.root}>
      {twitter && <Link to={twitter} icon={<Twitter />} />}
      {facebook && <Link to={facebook} icon={<Facebook />} />}
      {instagram && <Link to={instagram} icon={<Instagram />} />}
      {linkedIn && <Link to={linkedIn} icon={<LinkedIn />} />}
      {github && <Link to={github} icon={<Github />} />}
    </ul>
  );
};

export default SocialLinks;
