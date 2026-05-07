import React from 'react'
import { EmployeeRecord } from '../types'
import { buildRpgHomeSnapshot } from '../api/rpgHomeApi'

export function useRpgHomeState(employees: EmployeeRecord[]) {
  const snapshot = React.useMemo(() => buildRpgHomeSnapshot(employees), [employees])
  const [selectedEmployeeId, setSelectedEmployeeId] = React.useState<string | null>(null)

  const selectedEmployee = React.useMemo(
    () => snapshot.employees.find((actor) => actor.id === selectedEmployeeId) ?? null,
    [snapshot.employees, selectedEmployeeId],
  )

  return {
    snapshot,
    selectedEmployee,
    setSelectedEmployeeId,
  }
}
