import ExecutionEnvironment from '@docusaurus/ExecutionEnvironment';

import getCookie from '../utils/getCookie';

export default (function () {
  if (!ExecutionEnvironment.canUseDOM) {
    return null;
  }

  return {
    onRouteUpdate() {
      if ( getCookie(window.trackingCookieConsent) === 'true' ) {
        window.analytics.page();
      }
    },
  };
})();
