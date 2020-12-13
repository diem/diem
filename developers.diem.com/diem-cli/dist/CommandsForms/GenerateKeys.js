import React from 'react';
function GenerateKeys({ onSubmit }) {
    return (React.createElement("div", null,
        React.createElement("div", { className: "current-command" },
            React.createElement("form", { onSubmit: (e) => {
                    e.preventDefault();
                    onSubmit('generate_keys');
                } },
                React.createElement("h3", { className: "command-title" }, "Generate Keys"),
                React.createElement("p", null, "Generate new random keys pair."),
                React.createElement("button", { type: "submit" }, "Generate")))));
}
export default GenerateKeys;
