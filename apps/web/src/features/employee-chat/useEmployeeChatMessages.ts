import * as React from 'react'

export type ChatWireMessage = {
  id: string
  role: string
  content: string
  created_at_ms: number
}

export type ChatResultMeta = {
  exit_code: number
  tool_instance_id: string
  tool_kind: string
  model: string
  prompt_tokens: number
  completion_tokens: number
  total_tokens: number
}

type MessagesResponse = { messages: ChatWireMessage[] }
type PostMessageResponse = { messages: ChatWireMessage[]; last_result: ChatResultMeta }

export function useEmployeeChatMessages(apiBase: string, locale: string, employeeId: string | null) {
  const [serverMessages, setServerMessages] = React.useState<ChatWireMessage[]>([])
  const [loading, setLoading] = React.useState(false)
  const [sending, setSending] = React.useState(false)
  const [error, setError] = React.useState<string | null>(null)
  const [lastResult, setLastResult] = React.useState<ChatResultMeta | null>(null)

  const headers = React.useMemo(
    () => ({
      'x-lang': locale,
    }),
    [locale],
  )

  const refresh = React.useCallback(async () => {
    if (!employeeId) {
      setServerMessages([])
      setLastResult(null)
      setError(null)
      return
    }
    setLastResult(null)
    setLoading(true)
    setError(null)
    try {
      const url = `${apiBase}/api/employees/${encodeURIComponent(employeeId)}/messages`
      const res = await fetch(url, { headers })
      const text = await res.text()
      if (!res.ok) throw new Error(text || `HTTP ${res.status}`)
      const data = JSON.parse(text) as MessagesResponse
      setServerMessages(data.messages ?? [])
    } catch (e) {
      setServerMessages([])
      setError(e instanceof Error ? e.message : String(e))
    } finally {
      setLoading(false)
    }
  }, [apiBase, employeeId, headers])

  React.useEffect(() => {
    void refresh()
  }, [refresh])

  const sendMessage = React.useCallback(
    async (body: string) => {
      if (!employeeId || !body.trim()) return
      setSending(true)
      setError(null)
      try {
        const url = `${apiBase}/api/employees/${encodeURIComponent(employeeId)}/messages`
        const res = await fetch(url, {
          method: 'POST',
          headers: { ...headers, 'Content-Type': 'application/json' },
          body: JSON.stringify({ content: body }),
        })
        const text = await res.text()
        if (!res.ok) throw new Error(text || `HTTP ${res.status}`)
        const data = JSON.parse(text) as PostMessageResponse
        setServerMessages(data.messages ?? [])
        setLastResult(data.last_result ?? null)
      } catch (e) {
        setError(e instanceof Error ? e.message : String(e))
        throw e
      } finally {
        setSending(false)
      }
    },
    [apiBase, employeeId, headers],
  )

  return {
    serverMessages,
    loading,
    sending,
    error,
    lastResult,
    refresh,
    sendMessage,
  }
}
