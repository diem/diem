import React from 'react';

function GetTransactions({
  onSubmit,
}: {
  onSubmit: (command: string) => void;
}) {
  return (
    <div>
      <div className="current-command">
        <form
          onSubmit={(e) => {
            e.preventDefault();
            const formData = new FormData(e.currentTarget);
            onSubmit(
              `get_transactions --fromVersion ${formData.get(
                'fromVersion'
              )} --limit ${formData.get('limit')} --includeEvents${
                formData.get('includeEvents') ? '' : ' false'
              }${formData.get('prettify') ? ' --prettify' : ''}`
            );
          }}
        >
          <h3 className="command-title">Get Transaction</h3>
          <p>Fetch blockchain transactions in given version range.</p>
          <div className="form-group">
            <label>From Version</label>
            <input
              type="number"
              name="fromVersion"
              required
              className="form-typed-input"
            />

            <label>Limit</label>
            <input
              type="number"
              name="limit"
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

export default GetTransactions;
