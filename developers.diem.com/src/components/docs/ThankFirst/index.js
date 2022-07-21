import React from 'react';
import CardsWrapper from '../Cards/CardsWrapper';
import OverlayCard from '../Cards/OverlayCard';

const ThankFirst = () => {
  let description = (
    <span>Thanks to <a href="https://www.firstdag.com/">First DAG</a> for contributing to these projects</span>
  );

  return (
    <CardsWrapper cardsPerRow={2}>
      <OverlayCard
        to="https://www.firstdag.com/"
        icon="img/first-logo.svg"
        iconDark="img/first-logo-dark.svg"
        description={description}
      />
    </CardsWrapper>
  );
}

ThankFirst.propTypes = {};

export default ThankFirst;
