import React from 'react';

function AddressToBech32({
                           onSubmit
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
            onSubmit(`address_to_bech32 --address ${formData.get('address')} ${formData.get('subaddress') ? ` --subAddress ${formData.get('subaddress')}` : ''} --hrp tlb`);
          }}
        >
          <h3 className="command-title">Address To Bech32</h3>
          <p>Convert hexadecimal account address and subaddress to a Bech32 encoded address.</p>
          <div className="form-group">
            <label>Address</label>
            <input
              type="text"
              name="address"
              required
              className="form-typed-input"
            />
          </div>
          <div className="form-group">
            <label>Sub address</label>
            <input
              type="text"
              name="subaddress"
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

export default AddressToBech32;
