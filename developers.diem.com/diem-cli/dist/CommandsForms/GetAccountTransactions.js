import React from 'react';
function GetAccountTransactions({ onSubmit, }) {
    return (React.createElement("div", null,
        React.createElement("div", { className: "current-command" },
            React.createElement("form", { onSubmit: (e) => {
                    e.preventDefault();
                    const formData = new FormData(e.currentTarget);
                    onSubmit(`get_account_transactions --address ${formData.get('address')} --fromVersion ${formData.get('fromVersion')} --limit ${formData.get('limit')} --includeEvents${formData.get('includeEvents') ? '' : ' false'}${formData.get('prettify') ? ' --prettify' : ''}`);
                } },
                React.createElement("h3", { className: "command-title" }, "Get Account Transactions"),
                React.createElement("p", null, "Fetch a range of outgoing account transactions."),
                React.createElement("div", { className: "form-group" },
                    React.createElement("label", null, "Address"),
                    React.createElement("input", { type: "text", name: "address", required: true, className: "form-typed-input" }),
                    React.createElement("label", null, "From sequence number"),
                    React.createElement("input", { type: "number", name: "fromVersion", required: true, className: "form-typed-input" }),
                    React.createElement("label", null, "Limit"),
                    React.createElement("input", { type: "number", name: "limit", required: true, className: "form-typed-input" }),
                    React.createElement("div", { className: "form-check" },
                        React.createElement("input", { type: "checkbox", name: "includeEvents", className: "form-check-input", id: "checkIncludeEvents", defaultChecked: true }),
                        React.createElement("label", { className: "form-check-label", htmlFor: "checkIncludeEvents" }, "Include Events")),
                    React.createElement("div", { className: "form-check" },
                        React.createElement("input", { type: "checkbox", name: "prettify", className: "form-check-input", id: "checkPrettify", defaultChecked: true }),
                        React.createElement("label", { className: "form-check-label", htmlFor: "checkPrettify" }, "Prettify"))),
                React.createElement("button", { type: "submit" }, "Run")))));
}
export default GetAccountTransactions;
