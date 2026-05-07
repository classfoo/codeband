import { EmployeeRecord, RpgHomeSnapshot } from '../types'
import { createEmployeeActors } from '../actors/employeeFactory'
import { createPresidentActor } from '../actors/playerFactory'
import { officeScene } from '../scene/officeMap'

export function buildRpgHomeSnapshot(employees: EmployeeRecord[]): RpgHomeSnapshot {
  return {
    scene: officeScene,
    player: createPresidentActor(),
    employees: createEmployeeActors(employees),
  }
}
