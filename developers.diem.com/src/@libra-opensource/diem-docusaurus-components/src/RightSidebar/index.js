import React from "react";
import PropTypes from "prop-types";

import styles from "./styles.module.css";

import TOC from "./TOC";
import TopBar from "./TopBar";

const RightSidebar = (props) => {
  const { editUrl, headings } = props;
  return (
    <div className={styles.root}>
      <TopBar editUrl={editUrl} />
      <TOC headings={headings} />
    </div>
  );
};

RightSidebar.propTypes = {
  editUrl: PropTypes.string.isRequired,
  headings: PropTypes.array.isRequired,
};

RightSidebar.defaultProps = {
  headings: [],
};

export default RightSidebar;
