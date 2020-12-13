import React from 'react';
function CreateAccount({ onSubmit }) {
    return (React.createElement("div", null,
        React.createElement("div", { className: "current-command" },
            React.createElement("form", { onSubmit: (e) => {
                    e.preventDefault();
                    onSubmit('create_account');
                } },
                React.createElement("h3", { className: "command-title" }, "Create Account"),
                React.createElement("p", null, "Create a new blockchain account."),
                React.createElement("button", { type: "submit" }, "Create")))));
}
export default CreateAccount;
