import ExecutionEnvironment from '@docusaurus/ExecutionEnvironment';

import utils from 'libra-docusaurus-components/src/utils';

const {getCookie} = utils;

const areCookiesEnabled = () =>
  ExecutionEnvironment.canUseDOM
    ? getCookie(window.trackingCookieConsent) === 'true'
    : false;


export default {
  areCookiesEnabled,
};
