import React from 'react';
function GetAccount({ onSubmit }) {
    return (React.createElement("div", null,
        React.createElement("div", { className: "current-command" },
            React.createElement("form", { onSubmit: (e) => {
                    e.preventDefault();
                    const formData = new FormData(e.currentTarget);
                    onSubmit(`get_account --address ${formData.get('address')}`);
                } },
                React.createElement("h3", { className: "command-title" }, "Get Account"),
                React.createElement("p", null, "Fetch the latest account information for a given account address."),
                React.createElement("div", { className: "form-group" },
                    React.createElement("label", null, "Address"),
                    React.createElement("input", { type: "text", name: "address", required: true, className: "form-typed-input" })),
                React.createElement("button", { type: "submit" }, "Type in")))));
}
export default GetAccount;
