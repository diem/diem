import React from 'react';

function CreateAccount({ onSubmit }: { onSubmit: (command: string) => void }) {
  return (
    <div>
      <div className="current-command">
        <form
          onSubmit={(e) => {
            e.preventDefault();
            onSubmit('create_account');
          }}
        >
          <h3 className="command-title">Create Account</h3>
          <p>Create a new blockchain account.</p>
          <button type="submit" >
            Create
          </button>
        </form>
      </div>
    </div>
  );
}

export default CreateAccount;
