import React from 'react';

function GenerateKeys({ onSubmit }: { onSubmit: (command: string) => void }) {
  return (
    <div>
      <div className="current-command">
        <form
          onSubmit={(e) => {
            e.preventDefault();
            onSubmit('generate_keys');
          }}
        >
          <h3 className="command-title">Generate Keys</h3>
          <p>Generate new random keys pair.</p>
          <button type="submit" >
            Generate
          </button>
        </form>
      </div>
    </div>
  );
}

export default GenerateKeys;
