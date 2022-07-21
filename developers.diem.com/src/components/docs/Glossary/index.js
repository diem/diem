import React from 'react';
import PropTypes from 'prop-types';
import Link from "@docusaurus/Link";


function Glossary({children, slug = null}) {
  if (slug === null) {
    slug = children.toLowerCase().replace(/\W+/g, "");
  }
  return (
    <Link href={`/docs/reference/glossary#${slug}`}>text</Link>
  )
}

Glossary.propTypes = {
  children: PropTypes.string.isRequired,
  slug: PropTypes.string,
};

export default Glossary;
