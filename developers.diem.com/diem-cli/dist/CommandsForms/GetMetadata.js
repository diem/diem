import React from 'react';
function GetMetadata({ onSubmit }) {
    return (React.createElement("div", null,
        React.createElement("div", { className: "current-command" },
            React.createElement("form", { onSubmit: (e) => {
                    e.preventDefault();
                    const formData = new FormData(e.currentTarget);
                    if (formData.get('version')) {
                        onSubmit(`get_metadata --version ${formData.get('version')}`);
                    }
                    else {
                        onSubmit(`get_metadata`);
                    }
                } },
                React.createElement("h3", { className: "command-title" }, "Get Metadata"),
                React.createElement("p", null, "Get the Diem Blockchain metadata."),
                React.createElement("div", { className: "form-group" },
                    React.createElement("label", null, "Version"),
                    React.createElement("input", { type: "number", name: "version", className: "form-typed-input" })),
                React.createElement("button", { type: "submit" }, "Run")))));
}
export default GetMetadata;
