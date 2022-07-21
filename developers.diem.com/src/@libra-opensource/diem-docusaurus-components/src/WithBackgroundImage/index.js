import React, {useEffect, useState} from 'react';
import PropTypes from 'prop-types';

import useBaseUrl from '@docusaurus/useBaseUrl';
import useThemeContext from '@theme/hooks/useThemeContext';

let counter = 0;

const getImage = (
  [imageDark, imageDarkHover, imageLight, imageLightHover],
  isHovered,
) => {
  if (!imageLight) {
    return '';
  }

  const {isDarkTheme} = useThemeContext();

  const backgroundImage = isDarkTheme && imageDark ? imageDark : imageLight;

  let hoverBackgroundImage = isDarkTheme && imageDarkHover ? imageDarkHover : imageLightHover;
  hoverBackgroundImage = hoverBackgroundImage ? hoverBackgroundImage : backgroundImage;

  const image = isHovered ? hoverBackgroundImage : backgroundImage;

  return useBaseUrl(image);
};

// TODO (oliver): Simplify the passing in of images
const WithBackgroundImage = ({
  children,
  className,
  imageDark,
  imageDarkHover,
  imageLight,
  imageLightHover,
  tag: Tag,
  ...props
}) => {
  const [isHovered, setIsHovered] = useState(false);
  const images = [imageDark, imageDarkHover, imageLight, imageLightHover];

  const image = getImage(images, isHovered);

  const backgroundImageStyle = image
    ? {'backgroundImage': `url('${image}')`}
    : {};
  const imagesToPreload = images.filter(url => url).map(url => useBaseUrl(url));

  useEffect(() => {
    const preloadedImages = imagesToPreload.map(url => {
      const image = new Image();
      image.src = url;
      return image;
    });
    window.preloadedImages = window.preloadedImages || [];
    window.preloadedImages[counter++] = preloadedImages;
  }, []);

  return (
    <Tag
      className={className}
      onMouseEnter={() => setIsHovered(true)}
      onMouseLeave={() => setIsHovered(false)}
      style={backgroundImageStyle}
      {...props}
    >
      {children}
    </Tag>
  );
};

WithBackgroundImage.propTypes = {
  children: PropTypes.oneOfType([PropTypes.element, PropTypes.string]),
  imageDark: PropTypes.string,
  imageDarkHover: PropTypes.string,
  imageLight: PropTypes.string,
  imageLightHover: PropTypes.string,
  tag: PropTypes.oneOfType([PropTypes.string, PropTypes.func]),
};

WithBackgroundImage.defaultProps = {
  tag: 'div',
};

export default WithBackgroundImage;
