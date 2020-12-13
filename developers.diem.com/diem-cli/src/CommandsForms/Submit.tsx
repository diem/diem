import React from 'react';

function Submit({ onSubmit }: { onSubmit: (command: string) => void }) {
  return (
    <div>
      <div className="current-command">
        <form
          onSubmit={(e) => {
            e.preventDefault();
            onSubmit('submit');
          }}
        >
          <h3 className="command-title">Submit</h3>
          <button type="submit">Run</button>
        </form>
      </div>
    </div>
  );
}

export default Submit;
