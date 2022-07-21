import React from 'react';
import PropTypes from 'prop-types';
import Link from "../../Link";

const PublicationLink = ({image, doc_link, title}) => (
  <div>
    <p>
      <a href={doc_link} target="_blank">
        <img className="deep-dive-image" src={image} alt={`${title} PDF Download`}/>
      </a>
    </p>
    <Link href="publication-archive">Previous versions</Link>
  </div>
)

PublicationLink.propTypes = {
  image: PropTypes.string.isRequired,
  doc_link: PropTypes.string.isRequired,
  title: PropTypes.string.isRequired,
};

export default PublicationLink;
