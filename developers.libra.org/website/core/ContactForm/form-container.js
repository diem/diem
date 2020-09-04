/**
 * Copyright (c) The Libra Core Contributors
 * SPDX-License-Identifier: Apache-2.0
 */

const React = require('react');

const EnableJavaScript = require(`${process.cwd()}/core/ContactForm/enable-javascript.js`);
const FieldSet = require(`${process.cwd()}/core/ContactForm/fieldset.js`);
const FormHeader = require(`${process.cwd()}/core/ContactForm/form-header.js`);


const getFields = (fields) => {
  return fields.map((config, idx) => {
    return <FieldSet key={`form-fieldset-${idx}`} {...config} />;
  });
};

const getForm = (formId, fields) => {
  if (!formId) {
    return null;
  }

  return (
    <div className="formWrapper">
      <form id={formId}>
        {getFields(fields)}
        <div className="formControlGroup">
          <button className="button right" type="submit">Submit</button>
        </div>
      </form>
    </div>
  );
}


const FormContainer = (props) => {
  const {
    children,
    config: {
      baseUrl
    },
    fields,
    formId,
    title,
    subtitle
  } = props;

  /**
   * Get the className and src props for the img elements.
   */
  const getImageProps = (className, image) => {
    return {
      className: `formBgImage ${className}`,
      src: `${baseUrl}img/${image}`
    };
  };

  return (
    <div className="mainContainer formPage">
      <EnableJavaScript baseUrl={baseUrl} />
      <FormHeader title={title} subtitle={subtitle} />
      <div className="wrapper">
        <img {...getImageProps('formIcon', 'form-icon.svg')} />
        <img {...getImageProps('bgCircleLeft', 'bg-circle-whole.svg')} />
        <img {...getImageProps('bgCircleBottom', 'bg-circle-half.svg')} />
        <img {...getImageProps('bgCircleRight', 'bg-circle-whole.svg')} />
        <div className="mainContainer documentContainer postContainer">
          {getForm(formId, fields)}
          {children}
        </div>

      </div>

    </div>
  );
};

module.exports = FormContainer;
