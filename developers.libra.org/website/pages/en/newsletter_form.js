/**
 * Form for the Newletter signup.
 */
const React = require('react');

const FormContainer = require(`${process.cwd()}/core/ContactForm/form-container.js`);
const countryCodes = require(`${process.cwd()}/core/ContactForm/country-codes.js`);


const formFields = [{
  items: [{
    id: 'email',
    label: 'Email',
    type: 'email',
    required: true
  }, {
    id: 'interest',
    label: 'Interest',
    type: 'select',
    required: false,
    options: [{
      value: 'Protocol',
      text: 'Protocol'
    }, {
      value: 'Application development',
      text: 'Application development'
    }, {
      value: 'Both?',
      text: 'Both?'
    }]
  }, {
    id: 'background',
    label: 'Background',
    type: 'select',
    required: false,
    options: [{
      value: 'BlockchainDeveloper',
      text: 'Blockchain Developer'
    }, {
      value: 'Researcher',
      text: 'Researcher',
    }, {
      value: 'InstitutionalDeveloper',
      text: 'Institutional Developer'
    }, {
      value: 'dAppsDeveloper',
      text: 'dApps Developer'
    }, {
      value: 'Other',
      text: 'Other'
    }]
  }, {
    id: 'country',
    label: 'Country',
    type: 'select',
    required: false,
    options: countryCodes.map(country => {
      return {
        value: country.abbreviation,
        text: country.country
      };
    })
  }]
}];


function NewsletterForm(props) {
  return (
    <FormContainer
      {...props}
      fields={formFields}
      formId="newsletterForm"
      title="Newsletter Sign-up"
      subtitle="Please complete the form below and hit submit."
    />
  );
}

module.exports = NewsletterForm;
