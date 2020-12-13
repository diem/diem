import React from 'react';

function GetTransaction({ onSubmit }: { onSubmit: (command: string) => void }) {
  return (
    <div>
      <div className="current-command">
        <form
          onSubmit={(e) => {
            e.preventDefault();
            const formData = new FormData(e.currentTarget);
            onSubmit(
              `get_transaction --txVersion ${formData.get('txVersion')} --includeEvents${
                formData.get('includeEvents') ? '' : ' false'
              }${formData.get('prettify') ? ' --prettify' : ''}`
            );
          }}
        >
          <h3 className="command-title">Get Transaction</h3>
          <p>Fetch specific blockchain transaction.</p>
          <div className="form-group">
            <label>Version</label>
            <input
              type="number"
              name="txVersion"
              required
              className="form-typed-input"
            />

            <div className="form-check">
              <input
                type="checkbox"
                name="includeEvents"
                className="form-check-input"
                id="checkIncludeEvents"
                defaultChecked
              />
              <label className="form-check-label" htmlFor="checkIncludeEvents">
                Include Events
              </label>
            </div>

            <div className="form-check">
              <input
                type="checkbox"
                name="prettify"
                className="form-check-input"
                id="checkPrettify"
                defaultChecked
              />
              <label className="form-check-label" htmlFor="checkPrettify">
                Prettify
              </label>
            </div>
          </div>
          <button type="submit">Run</button>
        </form>
      </div>
    </div>
  );
}

export default GetTransaction;
