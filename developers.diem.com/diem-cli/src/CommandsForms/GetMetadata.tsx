import React from 'react';

function GetMetadata({ onSubmit }: { onSubmit: (command: string) => void }) {
  return (
    <div>
      <div className="current-command">
        <form
          onSubmit={(e) => {
            e.preventDefault();
            const formData = new FormData(e.currentTarget);
            if (formData.get('version')) {
              onSubmit(`get_metadata --version ${formData.get('version')}`);
            } else {
              onSubmit(`get_metadata`);
            }
          }}
        >
          <h3 className="command-title">Get Metadata</h3>
          <p>Get the Diem Blockchain metadata.</p>
          <div className="form-group">
            <label>Version</label>
            <input type="number" name="version" className="form-typed-input" />
          </div>
          <button type="submit">Run</button>
        </form>
      </div>
    </div>
  );
}

export default GetMetadata;
