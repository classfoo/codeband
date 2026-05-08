enum Direction {
  Up = 'up',
  Down = 'down',
  Left = 'left',
  Right = 'right',
}
import { RpgHomeSnapshot } from '../types'

type RuntimeOptions = {
  container: HTMLElement
  snapshot: RpgHomeSnapshot
  onInteractEmployee: (employeeActorId: string | null) => void
}

const PLAYER_COLOR = '#6ca8ff'
const EMPLOYEE_COLOR = '#7fdca5'
const WALL_COLOR = '#2b3242'
const GRID_COLOR = '#202736'

export class RpgRuntime {
  private readonly container: HTMLElement
  private readonly snapshot: RpgHomeSnapshot
  private readonly onInteractEmployee: (employeeActorId: string | null) => void
  private readonly canvas: HTMLCanvasElement
  private readonly context: CanvasRenderingContext2D
  private readonly pressed = new Set<string>()
  private animationId: number | null = null
  private readonly keydownListener: (event: KeyboardEvent) => void
  private readonly keyupListener: (event: KeyboardEvent) => void
  private player: { x: number; y: number }

  constructor(options: RuntimeOptions) {
    this.container = options.container
    this.snapshot = options.snapshot
    this.onInteractEmployee = options.onInteractEmployee
    this.canvas = document.createElement('canvas')
    this.canvas.width = this.snapshot.scene.width * this.snapshot.scene.tileSize
    this.canvas.height = this.snapshot.scene.height * this.snapshot.scene.tileSize
    this.canvas.className = 'rpg-home__canvas'
    this.context = this.canvas.getContext('2d') as CanvasRenderingContext2D
    this.keydownListener = (event) => this.handleKeyDown(event)
    this.keyupListener = (event) => this.handleKeyUp(event)
    this.player = { ...this.snapshot.player.position }
  }

  mount() {
    this.container.innerHTML = ''
    this.container.appendChild(this.canvas)
    window.addEventListener('keydown', this.keydownListener)
    window.addEventListener('keyup', this.keyupListener)
    this.loop()
  }

  destroy() {
    if (this.animationId !== null) {
      window.cancelAnimationFrame(this.animationId)
    }
    window.removeEventListener('keydown', this.keydownListener)
    window.removeEventListener('keyup', this.keyupListener)
    this.container.innerHTML = ''
  }

  private handleKeyDown(event: KeyboardEvent) {
    this.pressed.add(event.key.toLowerCase())
    if (event.key.toLowerCase() === 'e') {
      this.tryInteract()
    }
  }

  private handleKeyUp(event: KeyboardEvent) {
    this.pressed.delete(event.key.toLowerCase())
  }

  private loop = () => {
    this.updatePlayer()
    this.draw()
    this.animationId = window.requestAnimationFrame(this.loop)
  }

  private directionFromInput(): Direction | null {
    if (this.pressed.has('arrowup') || this.pressed.has('w')) return Direction.Up
    if (this.pressed.has('arrowdown') || this.pressed.has('s')) return Direction.Down
    if (this.pressed.has('arrowleft') || this.pressed.has('a')) return Direction.Left
    if (this.pressed.has('arrowright') || this.pressed.has('d')) return Direction.Right
    return null
  }

  private updatePlayer() {
    const direction = this.directionFromInput()
    if (direction === null) return
    const next = { ...this.player }
    if (direction === Direction.Up) next.y -= 1
    if (direction === Direction.Down) next.y += 1
    if (direction === Direction.Left) next.x -= 1
    if (direction === Direction.Right) next.x += 1

    const blocked = this.snapshot.scene.walls.some((wall) => wall.x === next.x && wall.y === next.y)
    if (!blocked) {
      this.player = next
    }
  }

  private tryInteract() {
    const target = this.snapshot.employees.find(
      (employee) =>
        Math.abs(employee.position.x - this.player.x) <= 1 &&
        Math.abs(employee.position.y - this.player.y) <= 1,
    )
    this.onInteractEmployee(target?.id ?? null)
  }

  private draw() {
    const { tileSize, width, height } = this.snapshot.scene
    this.context.clearRect(0, 0, this.canvas.width, this.canvas.height)
    this.context.fillStyle = '#151c29'
    this.context.fillRect(0, 0, this.canvas.width, this.canvas.height)

    this.context.strokeStyle = GRID_COLOR
    this.context.lineWidth = 1
    for (let x = 0; x <= width; x += 1) {
      this.context.beginPath()
      this.context.moveTo(x * tileSize, 0)
      this.context.lineTo(x * tileSize, height * tileSize)
      this.context.stroke()
    }
    for (let y = 0; y <= height; y += 1) {
      this.context.beginPath()
      this.context.moveTo(0, y * tileSize)
      this.context.lineTo(width * tileSize, y * tileSize)
      this.context.stroke()
    }

    this.snapshot.scene.zones.forEach((zone) => {
      this.context.fillStyle =
        zone.type === 'reception'
          ? '#2c4868'
          : zone.type === 'desk'
            ? '#355945'
            : zone.type === 'meeting'
              ? '#4d3f68'
              : '#6a5a3b'
      this.context.globalAlpha = 0.42
      this.context.fillRect(zone.x * tileSize, zone.y * tileSize, zone.width * tileSize, zone.height * tileSize)
      this.context.globalAlpha = 1
    })

    this.context.fillStyle = WALL_COLOR
    this.snapshot.scene.walls.forEach((wall) => {
      this.context.fillRect(wall.x * tileSize, wall.y * tileSize, tileSize, tileSize)
    })

    this.drawActor(this.player.x, this.player.y, PLAYER_COLOR)
    this.snapshot.employees.forEach((employee) => {
      this.drawActor(employee.position.x, employee.position.y, EMPLOYEE_COLOR)
    })
  }

  private drawActor(x: number, y: number, color: string) {
    const size = this.snapshot.scene.tileSize
    const centerX = x * size + size / 2
    const centerY = y * size + size / 2
    this.context.beginPath()
    this.context.fillStyle = color
    this.context.arc(centerX, centerY, size * 0.33, 0, Math.PI * 2)
    this.context.fill()
  }
}
