import React from 'react';
function AddressToBech32({ onSubmit }) {
    return (React.createElement("div", null,
        React.createElement("div", { className: "current-command" },
            React.createElement("form", { onSubmit: (e) => {
                    e.preventDefault();
                    const formData = new FormData(e.currentTarget);
                    onSubmit(`address_to_bech32 --address ${formData.get('address')} ${formData.get('subaddress') ? ` --subAddress ${formData.get('subaddress')}` : ''} --hrp tlb`);
                } },
                React.createElement("h3", { className: "command-title" }, "Address To Bech32"),
                React.createElement("p", null, "Convert hexadecimal account address and subaddress to a Bech32 encoded address."),
                React.createElement("div", { className: "form-group" },
                    React.createElement("label", null, "Address"),
                    React.createElement("input", { type: "text", name: "address", required: true, className: "form-typed-input" })),
                React.createElement("div", { className: "form-group" },
                    React.createElement("label", null, "Sub address"),
                    React.createElement("input", { type: "text", name: "subaddress", required: true, className: "form-typed-input" })),
                React.createElement("button", { type: "submit" }, "Type in")))));
}
export default AddressToBech32;
