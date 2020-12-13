import React from 'react';
import TabItem from '@theme-original/TabItem';

function Wrapper({ learnMoreLink, withinMultiStep, ...props }) {
  const newChild = React.cloneElement(props.children.props.children, {
    isWithinTab: true,
    learnMoreLink: learnMoreLink,
    withinMultiStep,
  });
  const newParent = React.cloneElement(props.children, {children: newChild});

  return <TabItem>{newParent}</TabItem>;
}

export default Wrapper;
