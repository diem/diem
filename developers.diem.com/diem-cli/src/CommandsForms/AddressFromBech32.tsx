import React from 'react';

function AddressFromBech32({
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
            onSubmit(`address_from_bech32 --address ${formData.get('encoded')}`
            );
          }}
        >
          <h3 className="command-title">Address From Bech32</h3>
          <p>Convert Bech32 encoded address to hexadecimal account address and subaddress.</p>
          <div className="form-group">
            <label>Encoded address</label>
            <input
              type="text"
              name="encoded"
              required
              className="form-typed-input"
            />
          </div>
          <button type="submit">Run</button>
        </form>
      </div>
    </div>
  );
}

export default AddressFromBech32;
