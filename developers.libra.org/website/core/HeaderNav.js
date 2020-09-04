const React = require('react');
const SubNav = require('./SubNav');

// header navbar used by all pages generated with docusaurus
class HeaderNav extends React.Component {
  makeHref(link) {
    if (link.href) {
      // set link to specified href
      return link.href;
    }
    if (link.doc) {
      // set link to document with current page's language/version
      const langPart = '';
      const versionPart = '';
      const id = langPart + versionPart + link.doc;
      return (
        this.props.config.baseUrl +
        this.props.getPath(this.props.metadata[id].permalink, this.props.config.cleanUrl)
      );
    }
    if (link.page) {
      // set link to page with current page's language if appropriate
      return this.props.config.baseUrl + link.page;
    }
    if (link.blog) {
      // set link to blog url
      return `${this.props.baseUrl}blog/`;
    }
  }
  // function to generate each header link, used with each object in this.props.config.headerLinks
  makeLinks(link) {
    const href = this.makeHref(link);
    let docItemActive = false;
    let docGroupActive = false;
    if (link.search && this.props.config.algolia) {
      // return algolia search bar
      const placeholder = this.props.config.algolia.placeholder || 'Search';
      return (
        <li className="navSearchWrapper reactNavSearchWrapper" key="search">
          <input
            id="search_input_react"
            type="text"
            placeholder={placeholder}
            title={placeholder}
          />
        </li>
      );
    }
    if (link.doc) {
      // set link to document with current page's language/version
      const langPart = '';
      const versionPart = '';
      const id = langPart + versionPart + link.doc;
      const { id: currentID, sidebar } = this.props.current;
      docItemActive = currentID && currentID === id;
      docGroupActive = sidebar && sidebar === this.props.metadata[id].sidebar;
    }
    const itemClasses = this.props.classNamesFn({
      siteNavGroupActive: (link.doc && docGroupActive) || (link.blog && this.props.current.blog),
      siteNavItemActive:
        docItemActive ||
        (link.blog && this.props.current.blogListing) ||
        (link.page && link.page === this.props.current.id) ||
        link.selected,
      highlight: link.highlight,
      'mobile-hidden': link.mobileMain,
    });
    const i18n = this.props.translation[this.props.language];
    return (
      <li key={`${link.label}page`} className={itemClasses}>
        <a href={href} target={link.external ? '_blank' : '_self'}>
          {this.props.idx(i18n, ['localized-strings', 'links', link.label]) || link.label}
        </a>
      </li>
    );
  }

  renderResponsiveNav() {
    const headerLinks = this.props.config.headerLinks;
    return (
      <div className="navigationWrapper navigationSlider mobile-hidden">
        <nav className="slidingNav">
          <ul className="nav-site nav-site-internal">{headerLinks.map(this.makeLinks, this)}</ul>
        </nav>
      </div>
    );
  }

  renderLogoContainer() {
    const headerClass = this.props.config.headerIcon ? 'headerTitleWithLogo' : 'headerTitle';
    const mainNavMobileLinks = this.props.config.headerLinks.filter((link) => link.mobileMain);
    return (
      <div className="logo-container">
        <div className="mobile pointer main-nav-trigger  trigger-open">
          <img src="/img/vertical-ellipse.svg" alt="open" />
        </div>
        <div className="mobile-hidden mobile pointer main-nav-trigger trigger-close">
          <img src="/img/close.svg" alt="close" />
        </div>
        <a href={this.props.config.headerLogoUrl || this.props.baseUrl}>
          {this.props.config.headerIcon && (
            <img
              className="logo"
              src={this.props.baseUrl + this.props.config.headerIcon}
              alt={this.props.config.title}
            />
          )}
          {!this.props.config.disableHeaderTitle && (
            <h2 className={headerClass}>{this.props.title}</h2>
          )}
        </a>
        <div className="mobile docs-image">
          {mainNavMobileLinks.map((link) => (
            <a key={`${link.label}page`} href={this.makeHref(link)}>
              {link.mobileImg ? (
                <img src={link.mobileImg.image} alt={link.mobileImg.alt || link.label} />
              ) : (
                link.label
              )}
            </a>
          ))}
        </div>
      </div>
    );
  }

  render() {
    return (
      <div>
        <div className="fixedHeaderContainer">
          <div className="headerWrapper wrapper">
            <header>
              <div className="main-nav">
                {this.renderLogoContainer()}
                {this.renderResponsiveNav()}
              </div>
            </header>
          </div>
          <SubNav {...this.props} />
        </div>
        <div className="navSpacer"></div>
      </div>
    );
  }
}

HeaderNav.defaultProps = {
  current: {},
};

module.exports = HeaderNav;
