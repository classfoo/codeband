import React from 'react'

export type EmployeeDirectoryRecord = {
  id: string
  name: string
  department: string
  role: string
  memory_file: string
}

type EmployeeListProps = {
  employees: EmployeeDirectoryRecord[]
  selectedEmployeeId: string | null
  onSelectEmployee: (id: string) => void
  t: (key: string) => string
}

export function EmployeeList({ employees, selectedEmployeeId, onSelectEmployee, t }: EmployeeListProps) {
  return (
    <div className="employee-list" role="listbox" aria-label={t('ui.employeeList.title')}>
      {employees.length === 0 ? (
        <div className="employee-list__empty">{t('ui.employeeList.empty')}</div>
      ) : (
        employees.map((item, index) => {
          const isActive = item.id === selectedEmployeeId
          const unread = index === 0
          const snippet = t('ui.employeeList.snippet').replace('{name}', item.name)
          const recentTime = unread ? '09:24' : t('ui.employeeList.yesterday')
          return (
            <button
              key={item.id}
              type="button"
              className={`employee-item ${isActive ? 'employee-item--active' : ''}`}
              onClick={() => onSelectEmployee(item.id)}
            >
              <div className="employee-item__avatar">{item.name.slice(0, 1).toUpperCase()}</div>
              <div className="employee-item__main">
                <div className="employee-item__name">{item.name}</div>
                <div className="employee-item__snippet">{snippet}</div>
              </div>
              <div className="employee-item__meta">
                <span className="employee-item__time">{recentTime}</span>
                <span
                  className={`employee-item__dot ${unread ? 'employee-item__dot--unread' : ''}`}
                  aria-label={unread ? t('ui.employeeList.unread') : t('ui.employeeList.read')}
                />
              </div>
            </button>
          )
        })
      )}
    </div>
  )
}
