/**
 * Copyright (c) The Libra Core Contributors
 * SPDX-License-Identifier: Apache-2.0
 */

const React = require('react');
const TextInput = require(`${process.cwd()}/core/ContactForm/text-input.js`);
const SelectInput = require(`${process.cwd()}/core/ContactForm/select-input.js`);


/**
 * Get the legend if "title" was passed in
 */
const getLegend = (title) => {
  return title ? (<legend>{title}</legend>) : null;
}

/**
 * Get the inputs from the config
 */
const getInputs = (items) => {
  return items.map(item => {
    // FIXME: Maybe include the class in the config. The goal is to
    // remove the need to parse a string.
    const Input = item.type === 'select' ? SelectInput : TextInput;
    return <Input key={`input-${item.id}`} {...item} />;
  });
}


const FieldSet = ({ title, items}) => {
  return (
    <fieldset>
      {getLegend(title)}
      {getInputs(items)}
    </fieldset>
  );
};

module.exports = FieldSet;
