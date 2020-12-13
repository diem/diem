import React from 'react';
function AddressFromBech32({ onSubmit, }) {
    return (React.createElement("div", null,
        React.createElement("div", { className: "current-command" },
            React.createElement("form", { onSubmit: (e) => {
                    e.preventDefault();
                    const formData = new FormData(e.currentTarget);
                    onSubmit(`address_from_bech32 --address ${formData.get('encoded')}`);
                } },
                React.createElement("h3", { className: "command-title" }, "Address From Bech32"),
                React.createElement("p", null, "Convert Bech32 encoded address to hexadecimal account address and subaddress."),
                React.createElement("div", { className: "form-group" },
                    React.createElement("label", null, "Encoded address"),
                    React.createElement("input", { type: "text", name: "encoded", required: true, className: "form-typed-input" })),
                React.createElement("button", { type: "submit" }, "Run")))));
}
export default AddressFromBech32;
