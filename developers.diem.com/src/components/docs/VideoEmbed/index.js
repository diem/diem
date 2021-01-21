import React from "react";
import PropTypes from 'prop-types';

import styles from './styles.module.css';

const VideoEmbed = ({ src }) => {
  return (
    <div className={styles.videoEmbed}>
      <iframe src={src} />
    </div>
  );
};

VideoEmbed.propTypes = {
  src: PropTypes.string.isRequired,
};

export default VideoEmbed;
