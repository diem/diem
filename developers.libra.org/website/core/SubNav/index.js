const React = require('react');

class SubNav extends React.Component {
  doesLinkMatchCurrentPage(link) {
    // set link to document with current page's language/version
    const langPart = '';
    const versionPart = '';
    const id = langPart + versionPart + link.doc;
    const { id: currentID, sidebar } = this.props.current;
    const isDocItemActive = currentID && currentID === id;
    const isDocGroupActive = link.doc && sidebar && sidebar === this.props.metadata[id].sidebar;
    return (
      (link.doc && isDocGroupActive) ||
      (link.doc && isDocItemActive) ||
      (link.blog && this.props.current.blog) ||
      (link.blog && this.props.current.blogListing) ||
      (link.page && link.page === this.props.current.id)
    );
  }

  renderBreadCrumb() {
    const breadcrumbs = this.props.config.subHeaderLinks.filter(
      this.doesLinkMatchCurrentPage,
      this,
    );
    return (
      <div className="breadcrumb">
        <h2>Developers</h2>
        {breadcrumbs.map(({ label }) => (
          <a key={`${label}breadcrumb`}>
            <span className="mobile-hidden">/</span>
            <span>{label}</span>
          </a>
        ))}
      </div>
    );
  }

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
  // function to generate each header link, used with each object in this.props.config.subHeaderLinks
  // This method just re-uses the makeLinks function described in HeaderNav (written by the original docusaurus team)
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
  renderResponsiveSubNav() {
    const subHeaderLinks = this.props.config.subHeaderLinks;
    return (
      <div className="navigationWrapper navigationSlider mobile-hidden">
        <nav className="slidingNav">
          <ul className="nav-site nav-site-internal">{subHeaderLinks.map(this.makeLinks, this)}</ul>
        </nav>
      </div>
    );
  }
  renderMobileTriggers() {
    return (
      <div>
        <div className="mobile sub-nav-trigger pointer trigger-open">
          <img src="/img/chevron-down.svg" alt="open" />
        </div>
        <div className="mobile-hidden mobile pointer sub-nav-trigger trigger-close">
          <img src="/img/chevron-pressed.svg" alt="close" />
        </div>
      </div>
    );
  }
  render() {
    return (
      <div id="SubNav">
        <header className="wrapper">
          {this.renderBreadCrumb()}
          {this.renderResponsiveSubNav()}
          {this.renderMobileTriggers()}
        </header>
      </div>
    );
  }
}

module.exports = SubNav;
