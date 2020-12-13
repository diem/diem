import React from 'react';
function GetCurrencies({ onSubmit }) {
    return (React.createElement("div", null,
        React.createElement("div", { className: "current-command" },
            React.createElement("form", { onSubmit: (e) => {
                    e.preventDefault();
                    onSubmit('get_currencies');
                } },
                React.createElement("h3", { className: "command-title" }, "Get Currencies"),
                React.createElement("p", null, "Get information about the currencies supported by the Diem Blockchain."),
                React.createElement("button", { type: "submit" }, "Run")))));
}
export default GetCurrencies;
