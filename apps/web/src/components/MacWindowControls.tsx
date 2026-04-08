import React from 'react'
import { getCurrentWindow } from '@tauri-apps/api/window'

type MacWindowControlsProps = {
  locale?: string
  t: (key: string) => string
}

export const MacWindowControls: React.FC<MacWindowControlsProps> = ({ t }) => {
  const appWindow = React.useMemo(() => getCurrentWindow(), [])

  const handleWindowControl = async (action: 'close' | 'minimize' | 'maximize') => {
    if (action === 'close') {
      await appWindow.close()
      return
    }
    if (action === 'minimize') {
      await appWindow.minimize()
      return
    }
    const maximized = await appWindow.isMaximized()
    if (maximized) {
      await appWindow.unmaximize()
    } else {
      await appWindow.maximize()
    }
  }

  return (
    <div className="mac-controls" onMouseDown={(e) => e.stopPropagation()}>
      <button
        className="mac-control mac-control--close"
        onClick={() => void handleWindowControl('close')}
        aria-label={t('ui.window.close')}
        title={t('ui.window.close')}
      />
      <button
        className="mac-control mac-control--minimize"
        onClick={() => void handleWindowControl('minimize')}
        aria-label={t('ui.window.minimize')}
        title={t('ui.window.minimize')}
      />
      <button
        className="mac-control mac-control--maximize"
        onClick={() => void handleWindowControl('maximize')}
        aria-label={t('ui.window.maximize')}
        title={t('ui.window.maximize')}
      />
    </div>
  )
}