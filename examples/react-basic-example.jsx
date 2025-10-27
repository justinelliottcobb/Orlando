import React, { useState } from 'react';
import { Pipeline } from 'orlando-transducers';

function App() {
  const [result, setResult] = useState(null);

  const handleProcess = () => {
    // Create a pipeline - WASM auto-initializes with bundler target
    const pipeline = new Pipeline()
      .map(x => x * 2)
      .filter(x => x > 10)
      .take(5);

    // Process data
    const data = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    const output = pipeline.toArray(data);

    setResult(output);
  };

  return (
    <div style={{ padding: '20px' }}>
      <h1>Orlando Transducers Demo</h1>

      <button onClick={handleProcess} style={{
        padding: '10px 20px',
        fontSize: '16px',
        background: '#007bff',
        color: 'white',
        border: 'none',
        borderRadius: '4px',
        cursor: 'pointer'
      }}>
        Process Data
      </button>

      {result && (
        <div style={{ marginTop: '20px' }}>
          <h3>Result:</h3>
          <pre style={{
            background: '#f5f5f5',
            padding: '15px',
            borderRadius: '4px'
          }}>
            {JSON.stringify(result, null, 2)}
          </pre>
        </div>
      )}
    </div>
  );
}

export default App;
