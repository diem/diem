import React from 'react';

function GetAccountTransaction({
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
              `get_account_transaction --address ${formData.get(
                'address'
              )} --sequenceNumber ${formData.get('sequenceNumber')} --includeEvents${
                formData.get('includeEvents') ? '' : ' false'
              }${formData.get('prettify') ? ' --prettify' : ''}`
            );
          }}
        >
          <h3 className="command-title">Get Account Transaction</h3>
          <p>Fetch an outgoing account transaction by its sequence number.</p>
          <div className="form-group">
            <label>Account address</label>
            <input
              type="text"
              name="address"
              required
              className="form-typed-input"
            />
          </div>
          <div className="form-group">
            <label>Sequence Number</label>
            <input
              type="number"
              name="sequenceNumber"
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

export default GetAccountTransaction;
