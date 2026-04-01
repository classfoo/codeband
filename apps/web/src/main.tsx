import React from 'react'
import ReactDOM from 'react-dom/client'

const API_BASE = import.meta.env.VITE_API_BASE ?? 'http://127.0.0.1:8080'

function App() {
  const [status, setStatus] = React.useState('checking...')

  React.useEffect(() => {
    fetch(`${API_BASE}/api/health`)
      .then((res) => res.json())
      .then((json) => setStatus(json.status ?? 'unknown'))
      .catch(() => setStatus('offline'))
  }, [])

  return (
    <main style={{ fontFamily: 'sans-serif', padding: 24 }}>
      <h1>Codeband</h1>
      <p>Backend status: {status}</p>
      <p>Desktop mode: run npm run dev:desktop</p>
      <p>Browser mode: run npm run dev:browser</p>
    </main>
  )
}

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
)
