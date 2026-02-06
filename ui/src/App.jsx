import { useState, useRef, useEffect } from 'react'
import { Send, Bot, User, Terminal } from 'lucide-react'
import ReactMarkdown from 'react-markdown'

function App() {
  const [messages, setMessages] = useState([
    { role: 'ai', text: 'Halo Mori! Noty siap memantau GitHub kamu. Ada yang bisa dibantu?' }
  ])
  const [input, setInput] = useState('')
  const [isLoading, setIsLoading] = useState(false)
  const messagesEndRef = useRef(null)

  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: "smooth" })
  }

  useEffect(() => {
    scrollToBottom()
  }, [messages])

  const sendMessage = async () => {
    if (!input.trim() || isLoading) return

    const userMsg = { role: 'user', text: input }
    setMessages(prev => [...prev, userMsg])
    setInput('')
    setIsLoading(true)

    try {
      // Tembak ke Backend Rust
      const response = await fetch('http://localhost:3000/api/chat', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ user_message: userMsg.text })
      })

      const data = await response.json()
      setMessages(prev => [...prev, { role: 'ai', text: data.reply }])
    } catch (error) {
      setMessages(prev => [...prev, { role: 'ai', text: "⚠️ Error: Backend Rust mati atau terblokir CORS." }])
    } finally {
      setIsLoading(false)
    }
  }

  return (
    <div className="flex flex-col h-screen max-w-4xl mx-auto bg-slate-900 shadow-2xl border-x border-slate-800 font-sans">
      
      {/* HEADER */}
      <div className="p-4 bg-slate-800 border-b border-slate-700 flex items-center justify-between sticky top-0 z-10">
        <div className="flex items-center gap-3">
          <div className="p-2 bg-indigo-600 rounded-lg shadow-lg shadow-indigo-500/20">
            <Terminal size={24} className="text-white" />
          </div>
          <div>
            <h1 className="font-bold text-lg text-white tracking-wide">Noty Assistant</h1>
            <p className="text-xs text-slate-400 flex items-center gap-1.5 font-medium">
              <span className="w-2 h-2 bg-green-500 rounded-full animate-pulse shadow-[0_0_8px_rgba(34,197,94,0.6)]"></span>
              System Active
            </p>
          </div>
        </div>
      </div>

      {/* CHAT AREA */}
      <div className="flex-1 overflow-y-auto p-4 space-y-6">
        {messages.map((msg, idx) => (
          <div key={idx} className={`flex gap-4 ${msg.role === 'user' ? 'flex-row-reverse' : ''}`}>
            
            {/* Avatar */}
            <div className={`w-10 h-10 rounded-full flex items-center justify-center shrink-0 shadow-lg
              ${msg.role === 'user' 
                ? 'bg-gradient-to-br from-indigo-500 to-purple-600' 
                : 'bg-gradient-to-br from-emerald-500 to-teal-600'}`}>
              {msg.role === 'user' ? <User size={18} className="text-white" /> : <Bot size={18} className="text-white" />}
            </div>

            {/* Bubble */}
            <div className={`p-4 rounded-2xl max-w-[85%] text-sm leading-7 shadow-md transition-all
              ${msg.role === 'user' 
                ? 'bg-indigo-600 text-white rounded-tr-none' 
                : 'bg-slate-800 text-slate-200 rounded-tl-none border border-slate-700'}`}>
              
              <ReactMarkdown 
                components={{
                  code: ({node, ...props}) => <code className="bg-black/30 px-1.5 py-0.5 rounded text-yellow-400 font-mono text-xs" {...props} />
                }}
              >
                {msg.text}
              </ReactMarkdown>
            </div>
          </div>
        ))}
        
        {isLoading && (
          <div className="flex gap-3 ml-1">
             <div className="w-10 h-10 rounded-full bg-slate-800 border border-slate-700 flex items-center justify-center animate-pulse">
               <Bot size={18} className="text-slate-500" />
             </div>
             <div className="text-slate-500 text-sm flex items-center italic">Noty is reviewing code...</div>
          </div>
        )}
        <div ref={messagesEndRef} />
      </div>

      {/* INPUT AREA */}
      <div className="p-4 bg-slate-800 border-t border-slate-700 pb-6">
        <div className="flex gap-2 relative">
          <input
            type="text"
            className="flex-1 bg-slate-900 border border-slate-700 text-white rounded-xl pl-5 pr-12 py-3.5 focus:outline-none focus:border-indigo-500 focus:ring-1 focus:ring-indigo-500 transition-all placeholder-slate-500"
            placeholder="Ketik pesan..."
            value={input}
            onChange={(e) => setInput(e.target.value)}
            onKeyDown={(e) => e.key === 'Enter' && sendMessage()}
            disabled={isLoading}
            autoFocus
          />
          <button 
            onClick={sendMessage}
            disabled={isLoading}
            className="absolute right-2 top-2 bottom-2 bg-indigo-600 hover:bg-indigo-700 text-white p-2.5 rounded-lg transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
          >
            <Send size={18} />
          </button>
        </div>
        <div className="text-center mt-2 text-[10px] text-slate-500">
          Powered by Rust & Gemini 3.0
        </div>
      </div>
    </div>
  )
}

export default App