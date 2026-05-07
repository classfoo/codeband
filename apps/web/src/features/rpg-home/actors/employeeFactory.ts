import { EmployeeRecord, RpgActor } from '../types'

const DESK_ANCHORS = [
  { x: 11, y: 3 },
  { x: 14, y: 3 },
  { x: 17, y: 3 },
  { x: 20, y: 3 },
  { x: 23, y: 3 },
  { x: 11, y: 6 },
  { x: 14, y: 6 },
  { x: 17, y: 6 },
  { x: 20, y: 6 },
  { x: 23, y: 6 },
]

export function createEmployeeActors(employees: EmployeeRecord[]): RpgActor[] {
  return employees.map((employee, index) => {
    const anchor = DESK_ANCHORS[index % DESK_ANCHORS.length]
    return {
      id: `employee-${employee.id}`,
      kind: 'employee',
      name: employee.name,
      department: employee.department,
      role: employee.role,
      position: anchor,
    }
  })
}
