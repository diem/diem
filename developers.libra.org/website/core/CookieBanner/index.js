const React = require('react');

const CookieBanner = () => {

  return (
    <aside id="cookie-banner">
      <div className="close" id="cookie-banner-close">
        <svg version="1.1" xmlns="http://www.w3.org/2000/svg">
          <line x1="1" y1="17"
                x2="17" y2="1"
                stroke="#fff"
                strokeWidth="2"/>
          <line x1="1" y1="1"
                x2="17" y2="17"
                stroke="#fff"
                strokeWidth="2"/>
        </svg>
      </div>
      <div className="wrapper">
        <p>To help us provide relevant content, analyze our traffic, and provide a variety of features, we use cookies. By clicking on or navigating the site, you agree to allow us to collect information on and off the Libra Website through cookies. To learn more, view our Cookie Policy:</p>
        <div className="buttonWrapper">
          <a className="button" href={'/docs/policies/cookies-policy'} target="_blank">
            Cookies Policy
          </a>
        </div>
      </div>
    </aside>
  );
}

module.exports = CookieBanner;
