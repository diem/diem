const path = require('path');

module.exports = function (context) {
  const {siteConfig} = context;
  const {customFields} = siteConfig;
  const {trackingCookieConsent, segment} = customFields || {};

  if (!segment) {
    throw new Error(
      `You need to specify 'segment' object in 'customFields' with 'productionKey' field in it to use diem-docusaurus-plugin-segment`,
    );
  }

  const {
    productionKey,
    stagingKey
  } = segment;

  if (!productionKey) {
    throw new Error(
      'You specified the `segment` object in `customFields` but the `productionKey` field was missing. ' +
        'Please ensure this is not a mistake.',
    );
  }

  const isProd = process.env.NODE_ENV === 'production';
  const apiKey = isProd && process.env.SEGMENT !== 'staging' ? productionKey : stagingKey;

  return {
    name: 'diem-docusaurus-plugin-segment',

    getClientModules() {
      return apiKey ? [path.resolve(__dirname, './analytics')] : [];
    },

    injectHtmlTags() {
      if (!apiKey) {
        return {};
      }
      return {
        headTags: [
          {
            tagName: 'script',
            innerHTML: `
              !function() {
                /*
                 * Necessary to allow the cookie name to be accessible as a variable
                 * from function components that don't use hooks (such as analytics.js)
                 */
                window.trackingCookieConsent = '${trackingCookieConsent}';

                window.loadSegmentFormScript = () => {
                  const segmentScript = document.createElement('script');
                  segmentScript.async = true;
                  segmentScript.src = '/js/segmentForm.js';
                  document.body.appendChild(segmentScript);

                  const formScript = document.createElement('script');
                  formScript.async = true;
                  formScript.src = '/js/forms.js';
                  document.body.appendChild(formScript);
                };

                /*
                 * We make the loading function accessible so that when the user accepts
                 * the cookie policy, segment is loaded without the need to refresh
                 */
                window.loadAnalytics = () => {
                  const cookieRow = document.cookie
                    .split('; ')
                    .find(row => row.startsWith('${trackingCookieConsent}='));
                  const cookiesEnabled = cookieRow ? cookieRow.split('=')[1] : 'false';

                  if (cookiesEnabled === 'true') {
                    var analytics=window.analytics=window.analytics||[];if(!analytics.initialize)if(analytics.invoked)window.console&&console.error&&console.error("Segment snippet included twice.");else{analytics.invoked=!0;analytics.methods=["trackSubmit","trackClick","trackLink","trackForm","pageview","identify","reset","group","track","ready","alias","debug","page","once","off","on","addSourceMiddleware","addIntegrationMiddleware","setAnonymousId","addDestinationMiddleware"];analytics.factory =function(e){return function(){var t=Array.prototype.slice.call(arguments);t.unshift(e);analytics.push(t);return analytics}};for(var e=0;e<analytics.methods.length;e++){var t=analytics.methods[e];analytics[t]=analytics.factory(t)}analytics.load=function(e,t){var n=document.createElement("script");n.type="text/javascript";n.async=!0;n.src="https://cdn.segment.com/analytics.js/v1/"+e+"/analytics.min.js";var a=document.getElementsByTagName("script")[0];a.parentNode.insertBefore(n,a);analytics._loadOptions=t};analytics.SNIPPET_VERSION="4.1.0"};
                    analytics.load("${apiKey}");
                    analytics.page();
                  }

                  if (window.isFormPage) {
                    window.loadSegmentFormScript();
                  }
                };

                window.loadAnalytics();
              }();
            `,
          },
        ],
      };
    },
  };
};
