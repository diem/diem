import React from 'react';

function GetCurrencies({ onSubmit }: { onSubmit: (command: string) => void }) {
  return (
    <div>
      <div className="current-command">
        <form
          onSubmit={(e) => {
            e.preventDefault();
            onSubmit('get_currencies');
          }}
        >
          <h3 className="command-title">Get Currencies</h3>
          <p>Get information about the currencies supported by the Diem Blockchain.</p>
          <button type="submit">Run</button>
        </form>
      </div>
    </div>
  );
}

export default GetCurrencies;
