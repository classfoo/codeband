import { EmployeeDirectoryRecord } from '../../components/EmployeeList'

export type RpgZoneType = 'desk' | 'meeting' | 'lounge' | 'reception'

export type GridPoint = { x: number; y: number }

export type RpgZone = {
  id: string
  type: RpgZoneType
  labelKey: string
  x: number
  y: number
  width: number
  height: number
}

export type RpgActorKind = 'president' | 'employee'

export type RpgActor = {
  id: string
  kind: RpgActorKind
  name: string
  department?: string
  role?: string
  position: GridPoint
}

export type RpgOfficeScene = {
  width: number
  height: number
  tileSize: number
  zones: RpgZone[]
  walls: GridPoint[]
}

export type RpgHomeSnapshot = {
  scene: RpgOfficeScene
  player: RpgActor
  employees: RpgActor[]
}

export type EmployeeRecord = EmployeeDirectoryRecord
