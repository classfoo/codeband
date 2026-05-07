import { RpgActor } from '../types'

export function createPresidentActor(): RpgActor {
  return {
    id: 'president',
    kind: 'president',
    name: 'President',
    role: 'president',
    position: { x: 3, y: 3 },
  }
}
