import React from 'react'

const API_BASE = import.meta.env.VITE_API_BASE ?? 'http://127.0.0.1:8080'
type WorkspaceStatus = {
  configured: boolean
  path: string | null
  source: 'env' | 'config' | 'unset'
}

export default function App() {
  const [status, setStatus] = React.useState('checking...')
  const [workspace, setWorkspace] = React.useState<WorkspaceStatus | null>(null)
  const [workspaceInput, setWorkspaceInput] = React.useState('')
  const [workspaceError, setWorkspaceError] = React.useState('')
  const [savingWorkspace, setSavingWorkspace] = React.useState(false)

  React.useEffect(() => {
    fetch(`${API_BASE}/api/health`)
      .then((res) => res.json())
      .then((json) => setStatus(json.status ?? 'unknown'))
      .catch(() => setStatus('offline'))

    fetch(`${API_BASE}/api/workspace`)
      .then((res) => res.json())
      .then((json: WorkspaceStatus) => {
        setWorkspace(json)
        if (json.path) {
          setWorkspaceInput(json.path)
        }
      })
      .catch(() => setWorkspaceError('Failed to load workspace status'))
  }, [])

  const saveWorkspace = React.useCallback(async () => {
    if (!workspaceInput.trim()) {
      setWorkspaceError('Workspace path cannot be empty')
      return
    }

    setSavingWorkspace(true)
    setWorkspaceError('')
    try {
      const response = await fetch(`${API_BASE}/api/workspace`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({ path: workspaceInput.trim() }),
      })
      if (!response.ok) {
        const errText = await response.text()
        throw new Error(errText || `HTTP ${response.status}`)
      }

      const data: WorkspaceStatus = await response.json()
      setWorkspace(data)
    } catch (err) {
      setWorkspaceError(
        err instanceof Error ? err.message : 'Failed to save workspace path',
      )
    } finally {
      setSavingWorkspace(false)
    }
  }, [workspaceInput])

  const needsWorkspaceSetup = workspace?.configured === false

  return (
    <div className="app-shell">
      <aside className="side-panel">
        <div className="side-panel__brand">Kaisha</div>
        <nav className="side-panel__nav">
          <button className="nav-item nav-item--active">Workspace</button>
          <button className="nav-item">Explorer</button>
          <button className="nav-item">Search</button>
          <button className="nav-item">Settings</button>
        </nav>
        <div className="side-panel__footer">
          <span>Backend</span>
          <span className={`status status--${status}`}>{status}</span>
        </div>
      </aside>

      <section className="work-area">
        <header className="work-area__topbar">
          <div className="topbar__title">
            {workspace?.configured ? 'Current Project' : 'Workspace Setup'}
          </div>
          <div className="topbar__actions">
            <button className="action-btn">Run</button>
            <button className="action-btn">Share</button>
          </div>
        </header>

        <main className="work-area__content">
          {needsWorkspaceSetup ? (
            <div className="workspace-setup">
              <h2 className="workspace-setup__title">Configure your work directory</h2>
              <p className="workspace-setup__hint">
                Set <code>KAISHA_WORKDIR</code> to force a path, or configure it
                below for first-time initialization.
              </p>
              <label className="workspace-setup__label" htmlFor="workspace-path">
                Workspace path
              </label>
              <input
                id="workspace-path"
                className="workspace-setup__input"
                value={workspaceInput}
                onChange={(event) => setWorkspaceInput(event.target.value)}
                placeholder="/path/to/kaisha-workspace"
              />
              <button
                className="action-btn workspace-setup__save"
                onClick={saveWorkspace}
                disabled={savingWorkspace}
              >
                {savingWorkspace ? 'Saving...' : 'Save Workspace'}
              </button>
              {workspaceError ? (
                <p className="workspace-setup__error">{workspaceError}</p>
              ) : null}
            </div>
          ) : (
            <div className="content-placeholder">
              <div>
                <div>Workspace</div>
                <div className="workspace-path">{workspace?.path ?? 'Not configured'}</div>
              </div>
            </div>
          )}
        </main>
      </section>
    </div>
  )
}
