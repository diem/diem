import React from 'react';

function GetEvents({ onSubmit }: { onSubmit: (command: string) => void }) {
  return (
    <div>
      <div className="current-command">
        <form
          onSubmit={(e) => {
            e.preventDefault();
            onSubmit('get_events');
          }}
        >
          <h3 className="command-title">Get Events</h3>
          <p>Fetch the events for a given event stream.</p>
          <button type="submit">Run</button>
        </form>
      </div>
    </div>
  );
}

export default GetEvents;
