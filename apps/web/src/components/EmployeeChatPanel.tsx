import React from 'react'
import { EmployeeDirectoryRecord } from './EmployeeList'

type ChatMessage = {
  id: string
  sender: 'employee' | 'me'
  content: string
  at: string
}

type EmployeeChatPanelProps = {
  employees: EmployeeDirectoryRecord[]
  selectedEmployeeId: string | null
  messageDraft: string
  onMessageDraftChange: (value: string) => void
  workspacePath: string | null
  t: (key: string) => string
}

function buildMessages(employee: EmployeeDirectoryRecord, t: (key: string) => string): ChatMessage[] {
  return [
    {
      id: `${employee.id}-1`,
      sender: 'employee',
      content: t('ui.chat.seed.employeeIntro').replace('{name}', employee.name),
      at: '09:20',
    },
    {
      id: `${employee.id}-2`,
      sender: 'me',
      content: t('ui.chat.seed.managerReply'),
      at: '09:23',
    },
  ]
}

export function EmployeeChatPanel({
  employees,
  selectedEmployeeId,
  messageDraft,
  onMessageDraftChange,
  workspacePath,
  t,
}: EmployeeChatPanelProps) {
  const selectedEmployee = employees.find((item) => item.id === selectedEmployeeId) ?? null
  const messages = selectedEmployee ? buildMessages(selectedEmployee, t) : []

  return (
    <div className="chat-layout">
      {selectedEmployee ? (
        <>
          <div className="chat-history">
            {messages.map((message) => (
              <div
                key={message.id}
                className={`chat-message ${message.sender === 'me' ? 'chat-message--me' : 'chat-message--employee'}`}
              >
                <div className="chat-message__content">{message.content}</div>
                <div className="chat-message__time">{message.at}</div>
              </div>
            ))}
          </div>
          <div className="chat-input-wrap">
            <div className="chat-toolbar chat-toolbar--top">
              <button className="action-btn">{t('ui.chat.toolbar.attach')}</button>
              <button className="action-btn">{t('ui.chat.toolbar.template')}</button>
            </div>
            <textarea
              className="chat-input"
              value={messageDraft}
              onChange={(event) => onMessageDraftChange(event.target.value)}
              placeholder={t('ui.chat.placeholder').replace('{name}', selectedEmployee.name)}
            />
            <div className="chat-toolbar chat-toolbar--bottom">
              <button className="action-btn">{t('ui.chat.toolbar.emoji')}</button>
              <button className="action-btn">{t('ui.chat.toolbar.send')}</button>
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
