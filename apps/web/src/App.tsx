import React from 'react'

const API_BASE = import.meta.env.VITE_API_BASE ?? 'http://127.0.0.1:8080'
type WorkspaceStatus = {
  configured: boolean
  path: string | null
  source: 'env' | 'config' | 'unset'
}
type SettingsSection = 'tools' | 'departments' | 'roles' | 'employees'
type ToolItem = { id: number; name: string; command: string; enabled: boolean }
type DepartmentItem = { id: number; name: string; lead: string }
type RoleItem = { id: number; name: string; level: 'junior' | 'mid' | 'senior' }
type EmployeeItem = { id: number; name: string; department: string; role: string }

export default function App() {
  const [status, setStatus] = React.useState('checking...')
  const [workspace, setWorkspace] = React.useState<WorkspaceStatus | null>(null)
  const [workspaceInput, setWorkspaceInput] = React.useState('')
  const [workspaceError, setWorkspaceError] = React.useState('')
  const [savingWorkspace, setSavingWorkspace] = React.useState(false)
  const [settingsOpen, setSettingsOpen] = React.useState(false)
  const [settingsSection, setSettingsSection] = React.useState<SettingsSection>('tools')
  const [toolForm, setToolForm] = React.useState({ name: '', command: '', enabled: true })
  const [departmentForm, setDepartmentForm] = React.useState({ name: '', lead: '' })
  const [roleForm, setRoleForm] = React.useState<RoleItem['level']>('mid')
  const [roleName, setRoleName] = React.useState('')
  const [employeeForm, setEmployeeForm] = React.useState({
    name: '',
    department: '',
    role: '',
  })
  const [tools, setTools] = React.useState<ToolItem[]>([])
  const [departments, setDepartments] = React.useState<DepartmentItem[]>([])
  const [roles, setRoles] = React.useState<RoleItem[]>([])
  const [employees, setEmployees] = React.useState<EmployeeItem[]>([])

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
  const nextId = React.useRef(1)

  const addTool = () => {
    if (!toolForm.name.trim() || !toolForm.command.trim()) return
    setTools((prev) => [
      ...prev,
      {
        id: nextId.current++,
        name: toolForm.name.trim(),
        command: toolForm.command.trim(),
        enabled: toolForm.enabled,
      },
    ])
    setToolForm({ name: '', command: '', enabled: true })
  }

  const addDepartment = () => {
    if (!departmentForm.name.trim() || !departmentForm.lead.trim()) return
    setDepartments((prev) => [
      ...prev,
      {
        id: nextId.current++,
        name: departmentForm.name.trim(),
        lead: departmentForm.lead.trim(),
      },
    ])
    setDepartmentForm({ name: '', lead: '' })
  }

  const addRole = () => {
    if (!roleName.trim()) return
    setRoles((prev) => [
      ...prev,
      {
        id: nextId.current++,
        name: roleName.trim(),
        level: roleForm,
      },
    ])
    setRoleName('')
    setRoleForm('mid')
  }

  const addEmployee = () => {
    if (!employeeForm.name.trim() || !employeeForm.department || !employeeForm.role) return
    setEmployees((prev) => [
      ...prev,
      {
        id: nextId.current++,
        name: employeeForm.name.trim(),
        department: employeeForm.department,
        role: employeeForm.role,
      },
    ])
    setEmployeeForm({ name: '', department: '', role: '' })
  }

  const renderSettingsCards = () => {
    if (settingsSection === 'tools') {
      return (
        <>
          <section className="settings-card">
            <h3 className="settings-card__title">Tool registration</h3>
            <div className="settings-grid">
              <input
                className="settings-input"
                placeholder="Tool name"
                value={toolForm.name}
                onChange={(event) => setToolForm((prev) => ({ ...prev, name: event.target.value }))}
              />
              <input
                className="settings-input"
                placeholder="Launch command"
                value={toolForm.command}
                onChange={(event) => setToolForm((prev) => ({ ...prev, command: event.target.value }))}
              />
              <label className="settings-checkbox">
                <input
                  type="checkbox"
                  checked={toolForm.enabled}
                  onChange={(event) => setToolForm((prev) => ({ ...prev, enabled: event.target.checked }))}
                />
                Enabled
              </label>
              <button className="action-btn" onClick={addTool}>Add Tool</button>
            </div>
          </section>
          <section className="settings-card">
            <h3 className="settings-card__title">Tool list</h3>
            {tools.length === 0 ? <p className="settings-empty">No tools configured.</p> : (
              <div className="settings-list">
                {tools.map((item) => (
                  <div key={item.id} className="settings-list__row">
                    <div>
                      <div>{item.name}</div>
                      <div className="settings-subtext">{item.command}</div>
                    </div>
                    <label className="settings-switch">
                      <input
                        type="checkbox"
                        checked={item.enabled}
                        onChange={(event) =>
                          setTools((prev) =>
                            prev.map((tool) =>
                              tool.id === item.id ? { ...tool, enabled: event.target.checked } : tool,
                            ),
                          )
                        }
                      />
                      <span>{item.enabled ? 'On' : 'Off'}</span>
                    </label>
                  </div>
                ))}
              </div>
            )}
          </section>
        </>
      )
    }

    if (settingsSection === 'departments') {
      return (
        <>
          <section className="settings-card">
            <h3 className="settings-card__title">Department setup</h3>
            <div className="settings-grid">
              <input
                className="settings-input"
                placeholder="Department name"
                value={departmentForm.name}
                onChange={(event) => setDepartmentForm((prev) => ({ ...prev, name: event.target.value }))}
              />
              <input
                className="settings-input"
                placeholder="Department lead"
                value={departmentForm.lead}
                onChange={(event) => setDepartmentForm((prev) => ({ ...prev, lead: event.target.value }))}
              />
              <button className="action-btn" onClick={addDepartment}>Add Department</button>
            </div>
          </section>
          <section className="settings-card">
            <h3 className="settings-card__title">Department list</h3>
            {departments.length === 0 ? <p className="settings-empty">No departments configured.</p> : (
              <div className="settings-list">
                {departments.map((item) => (
                  <div key={item.id} className="settings-list__row">
                    <div>{item.name}</div>
                    <div className="settings-subtext">Lead: {item.lead}</div>
                  </div>
                ))}
              </div>
            )}
          </section>
        </>
      )
    }

    if (settingsSection === 'roles') {
      return (
        <>
          <section className="settings-card">
            <h3 className="settings-card__title">Role setup</h3>
            <div className="settings-grid">
              <input
                className="settings-input"
                placeholder="Role name"
                value={roleName}
                onChange={(event) => setRoleName(event.target.value)}
              />
              <select
                className="settings-input"
                value={roleForm}
                onChange={(event) => setRoleForm(event.target.value as RoleItem['level'])}
              >
                <option value="junior">Junior</option>
                <option value="mid">Mid</option>
                <option value="senior">Senior</option>
              </select>
              <button className="action-btn" onClick={addRole}>Add Role</button>
            </div>
          </section>
          <section className="settings-card">
            <h3 className="settings-card__title">Role list</h3>
            {roles.length === 0 ? <p className="settings-empty">No roles configured.</p> : (
              <div className="settings-list">
                {roles.map((item) => (
                  <div key={item.id} className="settings-list__row">
                    <div>{item.name}</div>
                    <div className="settings-subtext">{item.level}</div>
                  </div>
                ))}
              </div>
            )}
          </section>
        </>
      )
    }

    return (
      <>
        <section className="settings-card">
          <h3 className="settings-card__title">Employee setup</h3>
          <div className="settings-grid">
            <input
              className="settings-input"
              placeholder="Employee name"
              value={employeeForm.name}
              onChange={(event) => setEmployeeForm((prev) => ({ ...prev, name: event.target.value }))}
            />
            <select
              className="settings-input"
              value={employeeForm.department}
              onChange={(event) => setEmployeeForm((prev) => ({ ...prev, department: event.target.value }))}
            >
              <option value="">Select department</option>
              {departments.map((item) => (
                <option key={item.id} value={item.name}>{item.name}</option>
              ))}
            </select>
            <select
              className="settings-input"
              value={employeeForm.role}
              onChange={(event) => setEmployeeForm((prev) => ({ ...prev, role: event.target.value }))}
            >
              <option value="">Select role</option>
              {roles.map((item) => (
                <option key={item.id} value={item.name}>{item.name}</option>
              ))}
            </select>
            <button className="action-btn" onClick={addEmployee}>Add Employee</button>
          </div>
        </section>
        <section className="settings-card">
          <h3 className="settings-card__title">Employee list</h3>
          {employees.length === 0 ? <p className="settings-empty">No employees configured.</p> : (
            <div className="settings-list">
              {employees.map((item) => (
                <div key={item.id} className="settings-list__row">
                  <div>{item.name}</div>
                  <div className="settings-subtext">{item.department} / {item.role}</div>
                </div>
              ))}
            </div>
          )}
        </section>
      </>
    )
  }

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
            <button className="action-btn" onClick={() => setSettingsOpen(true)}>Settings</button>
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
      {settingsOpen ? (
        <div className="settings-modal">
          <div className="settings-panel">
            <aside className="settings-sidebar">
              <div className="settings-sidebar__title">Configuration</div>
              <button
                className={`settings-nav ${settingsSection === 'tools' ? 'settings-nav--active' : ''}`}
                onClick={() => setSettingsSection('tools')}
              >
                Tools
              </button>
              <button
                className={`settings-nav ${settingsSection === 'departments' ? 'settings-nav--active' : ''}`}
                onClick={() => setSettingsSection('departments')}
              >
                Departments
              </button>
              <button
                className={`settings-nav ${settingsSection === 'roles' ? 'settings-nav--active' : ''}`}
                onClick={() => setSettingsSection('roles')}
              >
                Roles
              </button>
              <button
                className={`settings-nav ${settingsSection === 'employees' ? 'settings-nav--active' : ''}`}
                onClick={() => setSettingsSection('employees')}
              >
                Employees
              </button>
              <button className="action-btn settings-close" onClick={() => setSettingsOpen(false)}>
                Done
              </button>
            </aside>
            <section className="settings-content">
              {renderSettingsCards()}
            </section>
          </div>
        </div>
      ) : null}
    </div>
  )
}
