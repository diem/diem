import React, {useEffect} from 'react';
import ReactDOM from 'react-dom';

import Head from '@docusaurus/Head';
import isInternalUrl from '@docusaurus/isInternalUrl';
import AnnouncementBar from '@theme/AnnouncementBar';
import LayoutHead from '@theme/LayoutHead';
import LayoutProviders from '@theme/LayoutProviders';
import useBaseUrl from '@docusaurus/useBaseUrl';
import useKeyboardNavigation from '@theme/hooks/useKeyboardNavigation';

import CookieBanner from '../CookieBanner';
import CookieChoiceProvider from '../Contexts/CookieChoice/provider';
import Footer from '../Footer';
import Nav from '../Nav';

import classnames from 'classnames';
import styles from './styles.module.css';
import '../universal.css';

// Provided via plugins/react-axe-ada-monitoring
if (TEST_ADA) {
  var axe = require('react-axe');
  axe(React, ReactDOM, 1000);
}

function Layout(props) {
  const {
    containWidth = true,
    children,
    title,
    noFooter,
  } = props;

  useEffect(() => {
    if (window.location.hash) {
      const hashWithoutHash = window.location.hash.substring(1);
      document.getElementById(hashWithoutHash)?.scrollIntoView();
    }
  }, []);
  useKeyboardNavigation();

  return (
    <LayoutProviders>
      <CookieChoiceProvider>
        <LayoutHead {...props} />
        <AnnouncementBar />
        <div>
          <Nav />
          <div className={styles.navSpacer}></div>
        </div>
        <div className="nav-pusher">
          <div className={classnames("main-wrapper", {
            "width-wrapper": containWidth,
          })}>
            {children}
          </div>
          {!noFooter && <Footer />}
          <CookieBanner />
        </div>
      </CookieChoiceProvider>
    </LayoutProviders>
  );
}

export default Layout;
