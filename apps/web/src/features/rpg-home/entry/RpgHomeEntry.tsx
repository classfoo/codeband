import React from 'react'
import { EmployeeDirectoryRecord } from '../../../components/EmployeeList'
import { RpgRuntime } from '../engine/RpgRuntime'
import { useRpgHomeState } from '../state/useRpgHomeState'

type Props = {
  employees: EmployeeDirectoryRecord[]
  t: (key: string) => string
}

export function RpgHomeEntry({ employees, t }: Props) {
  const mountRef = React.useRef<HTMLDivElement | null>(null)
  const { snapshot, selectedEmployee, setSelectedEmployeeId } = useRpgHomeState(employees)

  React.useEffect(() => {
    if (!mountRef.current) return
    const runtime = new RpgRuntime({
      container: mountRef.current,
      snapshot,
      onInteractEmployee: setSelectedEmployeeId,
    })
    runtime.mount()
    return () => runtime.destroy()
  }, [snapshot, setSelectedEmployeeId])

  return (
    <section className="rpg-home">
      <header className="rpg-home__header">
        <h3 className="rpg-home__title">{t('ui.rpgHome.title')}</h3>
        <div className="rpg-home__hint">{t('ui.rpgHome.hint')}</div>
      </header>
      <div className="rpg-home__layout">
        <div className="rpg-home__viewport" ref={mountRef} />
        <aside className="rpg-home__sidebar">
          <div className="rpg-home__zone-list">
            <div>{t('ui.rpgHome.zone.reception')}</div>
            <div>{t('ui.rpgHome.zone.desk')}</div>
            <div>{t('ui.rpgHome.zone.meeting')}</div>
            <div>{t('ui.rpgHome.zone.lounge')}</div>
          </div>
          <div className="rpg-home__employee-card">
            {selectedEmployee ? (
              <>
                <div className="rpg-home__employee-name">{selectedEmployee.name}</div>
                <div className="rpg-home__employee-meta">
                  {selectedEmployee.department} / {selectedEmployee.role}
                </div>
              </>
            ) : (
              <div className="rpg-home__employee-empty">{t('ui.rpgHome.interactPrompt')}</div>
            )}
          </div>
        </aside>
      </div>
    </section>
  )
}
