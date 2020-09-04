/**
 * Copyright (c) The Libra Core Contributors
 * SPDX-License-Identifier: Apache-2.0
 */

const React = require('react');
const FormContainer = require(`${process.cwd()}/core/ContactForm/form-container.js`);
const FormHeader = require(`${process.cwd()}/core/ContactForm/form-header.js`);


const FormThanks = (props) => {
  return (
    <FormContainer
      {...props}
      title="Thank you!"
      subtitle=""
    >
      <a className="button" href={props.config.baseUrl}>Return</a>
    </FormContainer>
  );
};

module.exports = FormThanks;
