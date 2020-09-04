/**
 * Form for the Partner Interest data.
 */
const React = require('react');

const FormContainer = require(`${process.cwd()}/core/ContactForm/form-container.js`);


/**
 * NOTE: These ids should map to the Segment Zendesk mapping.
 * https://segment.com/docs/destinations/zendesk/
 *
 * See: static/js/forms.js. This is a stateless form and all
 * the dynamic aspects are in the forms.js file.
 */
const formFields = [{
  items: [{
    id: 'firstName',
    label: 'First name',
    type: 'text',
    maxLength: '50',
    required: true
  }, {
    id: 'lastName',
    label: 'Last name',
    type: 'text',
    maxLength: '50',
    required: true
  }, {
    id: 'email',
    label: 'Email address',
    type: 'email',
    maxLength: '250',
    required: true
  }, {
    id: 'companyId',
    label: 'Company name',
    type: 'text',
    maxLength: '250',
    required: true
  },{
    id: 'comments',
    label: 'Describe your goals in working with the Libra project (1000 characters max)',
    type: 'textarea',
    maxLength: '1000',
    rows: '5',
    required: true
  }]
}];

function DeveloperInterestForm(props) {
  return (
    <FormContainer
      {...props}
      formId="developerInterestForm_2020-04-17"
      fields={formFields}
      title="Developer Application Form"
      subtitle="Please complete the form below and hit submit."
    />
  );
}

module.exports = DeveloperInterestForm;
