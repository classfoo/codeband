import React from 'react'
import { useEmployeeChatMessages, type ChatWireMessage } from '../features/employee-chat/useEmployeeChatMessages'
import { EmployeeDirectoryRecord } from './EmployeeList'

type DisplayMessage = {
  id: string
  side: 'me' | 'employee' | 'system'
  content: string
  at: string
}

type EmployeeChatPanelProps = {
  apiBase: string
  locale: string
  employees: EmployeeDirectoryRecord[]
  selectedEmployeeId: string | null
  messageDraft: string
  onMessageDraftChange: (value: string) => void
  workspacePath: string | null
  t: (key: string) => string
}

function buildSeedMessages(employee: EmployeeDirectoryRecord, t: (key: string) => string): DisplayMessage[] {
  return [
    {
      id: `${employee.id}-seed-1`,
      side: 'employee',
      content: t('ui.chat.seed.employeeIntro').replace('{name}', employee.name),
      at: '09:20',
    },
    {
      id: `${employee.id}-seed-2`,
      side: 'me',
      content: t('ui.chat.seed.managerReply'),
      at: '09:23',
    },
  ]
}

function formatMessageTimeMs(ms: number): string {
  return new Date(ms).toLocaleTimeString(undefined, { hour: '2-digit', minute: '2-digit' })
}

function wireToDisplay(m: ChatWireMessage): DisplayMessage {
  const side = m.role === 'user' ? 'me' : m.role === 'system' ? 'system' : 'employee'
  return {
    id: m.id,
    side,
    content: m.content,
    at: formatMessageTimeMs(m.created_at_ms),
  }
}

export function EmployeeChatPanel({
  apiBase,
  locale,
  employees,
  selectedEmployeeId,
  messageDraft,
  onMessageDraftChange,
  workspacePath,
  t,
}: EmployeeChatPanelProps) {
  const selectedEmployee = employees.find((item) => item.id === selectedEmployeeId) ?? null
  const { serverMessages, loading, sending, error, lastResult, sendMessage } = useEmployeeChatMessages(
    apiBase,
    locale,
    selectedEmployeeId,
  )

  const historyRef = React.useRef<HTMLDivElement>(null)

  const displayMessages = React.useMemo((): DisplayMessage[] => {
    if (!selectedEmployee) return []
    if (serverMessages.length > 0) {
      return serverMessages.map(wireToDisplay)
    }
    return buildSeedMessages(selectedEmployee, t)
  }, [selectedEmployee, serverMessages, t])

  React.useEffect(() => {
    const el = historyRef.current
    if (!el) return
    el.scrollTop = el.scrollHeight
  }, [displayMessages, sending])

  const sendFromDraft = React.useCallback(async () => {
    if (!selectedEmployeeId || !selectedEmployee) return
    const text = messageDraft.trim()
    if (!text) return
    try {
      await sendMessage(text)
      onMessageDraftChange('')
    } catch {
      /* error surfaced via hook */
    }
  }, [messageDraft, onMessageDraftChange, selectedEmployee, selectedEmployeeId, sendMessage])

  const handlePromptKeyDown = React.useCallback(
    (event: React.KeyboardEvent<HTMLTextAreaElement>) => {
      if (event.key !== 'Enter') return
      if (event.shiftKey) return
      if (event.nativeEvent.isComposing) return
      event.preventDefault()
      void sendFromDraft()
    },
    [sendFromDraft],
  )

  const messageClass = (side: DisplayMessage['side']) => {
    if (side === 'me') return 'chat-message chat-message--me'
    if (side === 'system') return 'chat-message chat-message--system'
    return 'chat-message chat-message--employee'
  }

  return (
    <div className="chat-layout">
      {selectedEmployee ? (
        <>
          <div className="chat-history" ref={historyRef}>
            {loading ? <div className="chat-history__status">{t('ui.chat.loadingHistory')}</div> : null}
            {!loading && lastResult && lastResult.exit_code !== 0 ? (
              <div className="chat-history__warn">
                {t('ui.chat.toolExitWarning').replace('{code}', String(lastResult.exit_code))}
              </div>
            ) : null}
            {displayMessages.map((message) => (
              <div key={message.id} className={messageClass(message.side)}>
                <div className="chat-message__content">{message.content}</div>
                <div className="chat-message__time">{message.at}</div>
              </div>
            ))}
            {sending ? <div className="chat-history__status">{t('ui.chat.sending')}</div> : null}
          </div>
          {!loading && error ? (
            <div className="chat-inline-status chat-inline-status--error">
              {error}
            </div>
          ) : null}
          <div className="chat-input-wrap">
            <div className="chat-toolbar chat-toolbar--top">
              <button type="button" className="action-btn">
                {t('ui.chat.toolbar.attach')}
              </button>
              <button type="button" className="action-btn">
                {t('ui.chat.toolbar.template')}
              </button>
            </div>
            <textarea
              className="chat-input"
              value={messageDraft}
              onChange={(event) => onMessageDraftChange(event.target.value)}
              onKeyDown={handlePromptKeyDown}
              disabled={sending}
              placeholder={t('ui.chat.placeholder').replace('{name}', selectedEmployee.name)}
            />
            <div className="chat-toolbar chat-toolbar--bottom">
              <button type="button" className="action-btn" disabled={sending}>
                {t('ui.chat.toolbar.emoji')}
              </button>
              <button type="button" className="action-btn" onClick={() => void sendFromDraft()} disabled={sending}>
                {t('ui.chat.toolbar.send')}
              </button>
            </div>
          </div>
        </>
      ) : (
        <div className="content-placeholder">
          <div>{t('ui.chat.emptySelection')}</div>
        </div>
      )}
      <div className="workspace-path">{workspacePath ?? t('ui.workspace.notConfigured')}</div>
    </div>
  )
}
