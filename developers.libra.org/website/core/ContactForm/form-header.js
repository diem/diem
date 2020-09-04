/**
 * Copyright (c) The Libra Core Contributors
 * SPDX-License-Identifier: Apache-2.0
 */

const React = require('react');


const FormHeader = ({ title, subtitle }) => {
  return (
    <header className="formHero">
      <h1>{title}</h1>
      <h2>{subtitle}</h2>
    </header>
  );
};

module.exports = FormHeader;
