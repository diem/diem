import React from 'react';
function Submit({ onSubmit }) {
    return (React.createElement("div", null,
        React.createElement("div", { className: "current-command" },
            React.createElement("form", { onSubmit: (e) => {
                    e.preventDefault();
                    onSubmit('submit');
                } },
                React.createElement("h3", { className: "command-title" }, "Submit"),
                React.createElement("button", { type: "submit" }, "Run")))));
}
export default Submit;
