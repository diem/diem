import React from 'react';
function GetEvents({ onSubmit }) {
    return (React.createElement("div", null,
        React.createElement("div", { className: "current-command" },
            React.createElement("form", { onSubmit: (e) => {
                    e.preventDefault();
                    onSubmit('get_events');
                } },
                React.createElement("h3", { className: "command-title" }, "Get Events"),
                React.createElement("p", null, "Fetch the events for a given event stream."),
                React.createElement("button", { type: "submit" }, "Run")))));
}
export default GetEvents;
