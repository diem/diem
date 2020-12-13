import React from 'react';

function GetAccount({ onSubmit }: { onSubmit: (command: string) => void }) {
  return (
    <div>
      <div className="current-command">
        <form
          onSubmit={(e) => {
            e.preventDefault();
            const formData = new FormData(e.currentTarget);
            onSubmit(`get_account --address ${formData.get('address')}`);
          }}
        >
          <h3 className="command-title">Get Account</h3>
          <p>Fetch the latest account information for a given account address.</p>
          <div className="form-group">
            <label>Address</label>
            <input
              type="text"
              name="address"
              required
              className="form-typed-input"
            />
          </div>
          <button type="submit" >
            Type in
          </button>
        </form>
      </div>
    </div>
  );
}

export default GetAccount;
