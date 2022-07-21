import React, {useState} from 'react';

import CookieChoiceContext from './context';

import utils from '../../utils/index.js';

const {areTrackingCookiesEnabled} = utils;

const Provider = ({children}) => {
  const [cookiesEnabled, setCookiesEnabled] = useState(areTrackingCookiesEnabled);

  return (
    <CookieChoiceContext.Provider value={{
      cookiesEnabled, setCookiesEnabled,
    }}>
      {children}
    </CookieChoiceContext.Provider>
  );
}

export default Provider;
