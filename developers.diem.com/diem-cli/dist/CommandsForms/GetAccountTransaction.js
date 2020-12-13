import React from 'react';
function GetAccountTransaction({ onSubmit, }) {
    return (React.createElement("div", null,
        React.createElement("div", { className: "current-command" },
            React.createElement("form", { onSubmit: (e) => {
                    e.preventDefault();
                    const formData = new FormData(e.currentTarget);
                    onSubmit(`get_account_transaction --address ${formData.get('address')} --sequenceNumber ${formData.get('sequenceNumber')} --includeEvents${formData.get('includeEvents') ? '' : ' false'}${formData.get('prettify') ? ' --prettify' : ''}`);
                } },
                React.createElement("h3", { className: "command-title" }, "Get Account Transaction"),
                React.createElement("p", null, "Fetch an outgoing account transaction by its sequence number."),
                React.createElement("div", { className: "form-group" },
                    React.createElement("label", null, "Account address"),
                    React.createElement("input", { type: "text", name: "address", required: true, className: "form-typed-input" })),
                React.createElement("div", { className: "form-group" },
                    React.createElement("label", null, "Sequence Number"),
                    React.createElement("input", { type: "number", name: "sequenceNumber", required: true, className: "form-typed-input" }),
                    React.createElement("div", { className: "form-check" },
                        React.createElement("input", { type: "checkbox", name: "includeEvents", className: "form-check-input", id: "checkIncludeEvents", defaultChecked: true }),
                        React.createElement("label", { className: "form-check-label", htmlFor: "checkIncludeEvents" }, "Include Events")),
                    React.createElement("div", { className: "form-check" },
                        React.createElement("input", { type: "checkbox", name: "prettify", className: "form-check-input", id: "checkPrettify", defaultChecked: true }),
                        React.createElement("label", { className: "form-check-label", htmlFor: "checkPrettify" }, "Prettify"))),
                React.createElement("button", { type: "submit" }, "Run")))));
}
export default GetAccountTransaction;
