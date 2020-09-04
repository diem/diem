/**
 * Copyright (c) The Libra Core Contributors
 * SPDX-License-Identifier: Apache-2.0
 */

const PropTypes = require('prop-types');
const React = require('react');
const CookieBanner = require(`${process.cwd()}/core/CookieBanner`);

function SocialFooter(props) {
  const projectName = 'libra';
  const repoUrl = `https://github.com/${props.config.organizationName}/${
    projectName
  }`;
  const baseUrl = props.config.baseUrl;

  return (
    <div className="footerSection">
      <h5>Social</h5>
      <div className="social">
        <a
          className="github-button" // part of the https://buttons.github.io/buttons.js script in siteConfig.js
          href={repoUrl}
          data-count-href={`${repoUrl}/stargazers`}
          data-show-count="true"
          data-count-aria-label="# stargazers on GitHub"
          aria-label="Star Libra on GitHub">
          {projectName}
        </a>
      </div>
      <div className="social">
        <a
          href={"https://twitter.com/libradev?ref_src=twsrc%5Etfw"}
          className={"twitter-follow-button"}
          data-show-count={false}>
            Follow @libradev
        </a>
        <script
          async
          src={`${baseUrl}js/twitter-widgets.js`}
          charSet={"utf-8"}
        />
      </div>
    </div>
  );
}

SocialFooter.propTypes = {
  config: PropTypes.object,
};

class Footer extends React.Component {
  docUrl(doc, language) {
    const baseUrl = this.props.config.baseUrl;
    const docsUrl = this.props.config.docsUrl;
    const docsPart = `${docsUrl ? `${docsUrl}/` : ''}`;
    const langPart = `${language ? `${language}/` : ''}`;
    return `${baseUrl}${docsPart}${langPart}${doc}`;
  }

  pageUrl(doc, language) {
    const baseUrl = this.props.config.baseUrl;
    return baseUrl + (language ? `${language}/` : '') + doc;
  }

  render() {
    const currentYear = new Date().getFullYear();
    return (
      <footer className="nav-footer" id="footer">
        <section className="sitemap">
          {this.props.config.footerIcon && (
            <a href={this.props.config.baseUrl} className="nav-home">
              <img
                src={`${this.props.config.baseUrl}${
                  this.props.config.footerIcon
                }`}
                alt={this.props.config.title}
              />
            </a>
          )}
          <div className="footerSection">
            <h5>Learn About Libra</h5>
            <a href={this.docUrl('welcome-to-libra')}>Welcome to Libra</a>
            <a href={this.docUrl('libra-protocol')}>Libra Protocol</a>
            <a href={this.docUrl('the-libra-blockchain-paper')}>Libra Blockchain</a>
            <a href={this.docUrl('life-of-a-transaction')}>Life of a Transaction</a>
            <p />
            <h5>Try Libra Core </h5>
            <a href={this.docUrl('my-first-transaction')}>My First Transaction</a>
            <a href={this.docUrl('move-overview')}>Getting Started With Move</a>
          </div>
          <div className="footerSection">
            <h5>Policies</h5>
            <a href={this.docUrl('policies/privacy-policy')}>Privacy Policy</a>
            <a href={this.docUrl('policies/terms-of-use')}>Terms of Use</a>
            <a href={this.docUrl('policies/cookies-policy')}>Cookies Policy</a>
            <a href={this.docUrl('policies/code-of-conduct')}>Code of Conduct</a>
            <p />
            <h5>Community</h5>
            <a href="https://community.libra.org/">Developer Forum</a>
            <a href="https://developers.libra.org/newsletter_form">Newsletter</a>
          </div>
          <SocialFooter config={this.props.config} />
        </section>
        <section className="copyright">
          {this.props.config.copyright && (
            <span>{this.props.config.copyright}</span>
          )}{' '}
          &copy; Libra Association
        </section>
        <CookieBanner />
      </footer>
    );
  }
}

module.exports = Footer;
