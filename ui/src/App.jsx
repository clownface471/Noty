import { useState, useEffect } from 'react'
import { 
  Book, Brain, Settings, Plus, Calendar, Clock, Tag, 
  Trash2, Search, Filter, Smile, Terminal, Save, CheckCircle, Eye, Edit2,
  Github, ToggleLeft, ToggleRight
} from 'lucide-react'
import ReactMarkdown from 'react-markdown'

function App() {
  const [view, setView] = useState('timeline') 
  const [logs, setLogs] = useState([])
  const [personas, setPersonas] = useState([])
  const [loading, setLoading] = useState(false)
  const [editorMode, setEditorMode] = useState('write') // 'write' | 'preview'
  

const [githubConfig, setGithubConfig] = useState({
  repo_name: '',
  token: '',
  is_active: false,
  is_token_set: false
})

  // --- STATE SETTINGS ---
  const [settingsConfig, setSettingsConfig] = useState({
    username: '',
    ai_model_name: 'gemini-1.5-flash',
    ai_api_key: '',
    is_key_set: false
  })
  const [msg, setMsg] = useState('')

  // --- STATE NEW ENTRY ---
  const [newEntry, setNewEntry] = useState({
    content: '',
    entry_date: new Date().toISOString().split('T')[0],
    entry_time: new Date().toTimeString().split(' ')[0],
    tags: '',
    category: 'General',
    mood: 'Neutral'
  })

  // --- API FETCHERS ---
  const fetchLogs = async () => {
    try {
      const res = await fetch('http://localhost:3000/api/logs')
      setLogs(await res.json())
    } catch (e) { console.error(e) }
  }

  const fetchPersonas = async () => {
    try {
      const res = await fetch('http://localhost:3000/api/personas')
      setPersonas(await res.json())
    } catch (e) { console.error(e) }
  }

const fetchGithubConfig = async () => {
  try {
    const res = await fetch('http://localhost:3000/api/integrations/github')
    const data = await res.json()
    setGithubConfig({
      repo_name: data.repo_name,
      token: '', // token asli tidak ditampilkan
      is_active: data.is_active,
      is_token_set: data.is_token_set
    })
  } catch (e) { console.error(e) }
}

const fetchSettings = async () => {
    try {
      const res = await fetch('http://localhost:3000/api/settings')
      const data = await res.json()
      setSettingsConfig({
        username: data.username,
        // FIX: Kalau data dari DB kosong, langsung isi default 'gemini-3-flash-preview'
        ai_model_name: data.ai_model_name || 'gemini-3-flash-preview', 
        ai_api_key: '', 
        is_key_set: data.is_api_key_set
      })
    } catch (e) { console.error(e) }
  }

  // Initial Load
  useEffect(() => {
    fetchLogs()
    fetchPersonas()
  }, [])

  // Load Settings saat masuk menu Settings
  useEffect(() => {
    if (view === 'settings') fetchSettings()
      fetchGithubConfig()
  }, [view])

  // --- ACTIONS ---

  const saveGithub = async () => {
  setLoading(true)
  try {
    await fetch('http://localhost:3000/api/integrations/github', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(githubConfig)
    })
    setMsg('Integrasi GitHub Tersimpan! ✅')
    fetchGithubConfig()
    setTimeout(() => setMsg(''), 3000)
  } catch (e) { setMsg('Gagal simpan GitHub ❌') }
  finally { setLoading(false) }
}

  const saveSettings = async () => {
    setLoading(true)
    try {
      await fetch('http://localhost:3000/api/settings', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(settingsConfig)
      })
      setMsg('Pengaturan tersimpan! ✅')
      fetchSettings() // Refresh status key
      setTimeout(() => setMsg(''), 3000)
    } catch (e) { setMsg('Gagal simpan ❌') }
    finally { setLoading(false) }
  }

  const activatePersona = async (id) => {
    try {
      await fetch(`http://localhost:3000/api/personas/${id}/activate`, { method: 'POST' })
      fetchPersonas()
    } catch (e) { console.error(e) }
  }

  const submitLog = async () => {
    if (!newEntry.content) return
    setLoading(true)
    const tagsArray = newEntry.tags.split(',').map(t => t.trim()).filter(t => t)

    try {
      await fetch('http://localhost:3000/api/logs', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ ...newEntry, tags: tagsArray })
      })
      setView('timeline')
      fetchLogs()
      setNewEntry({
        content: '',
        entry_date: new Date().toISOString().split('T')[0],
        entry_time: new Date().toTimeString().split(' ')[0],
        tags: '',
        category: 'General',
        mood: 'Neutral'
      })
    } catch (e) { alert("Gagal menyimpan log") }
    finally { setLoading(false) }
  }

  const deleteLog = async (id) => {
    if (!confirm("Hapus catatan ini?")) return
    try {
      await fetch(`http://localhost:3000/api/logs/${id}`, { method: 'DELETE' })
      fetchLogs()
    } catch (e) { console.error(e) }
  }

  // --- RENDER ---
  const SidebarItem = ({ icon: Icon, label, active, onClick }) => (
    <button onClick={onClick} className={`w-full flex items-center gap-3 p-3 rounded-xl transition-all ${active ? 'bg-indigo-600 text-white shadow-lg' : 'text-slate-400 hover:bg-slate-800 hover:text-white'}`}>
      <Icon size={20} /> <span className="font-medium text-sm">{label}</span>
    </button>
  )

  return (
    <div className="flex h-screen bg-slate-950 text-slate-200 font-sans overflow-hidden">
      
      {/* SIDEBAR */}
      <div className="w-64 bg-slate-900 border-r border-slate-800 p-4 flex flex-col">
        <div className="flex items-center gap-3 mb-8 px-2">
          <div className="p-2 bg-gradient-to-br from-indigo-500 to-purple-600 rounded-lg">
            <Book size={24} className="text-white" />
          </div>
          <h1 className="font-bold text-xl tracking-tight text-white">Noty v2</h1>
        </div>
        <nav className="space-y-2 flex-1">
          <SidebarItem icon={Book} label="Logbook Timeline" active={view === 'timeline'} onClick={() => setView('timeline')} />
          <SidebarItem icon={Brain} label="AI Personas" active={view === 'personas'} onClick={() => setView('personas')} />
          <SidebarItem icon={Settings} label="Settings" active={view === 'settings'} onClick={() => setView('settings')} />
        </nav>
        <div className="mt-auto pt-4 border-t border-slate-800">
          <button onClick={() => setView('new-entry')} className="w-full bg-emerald-600 hover:bg-emerald-700 text-white p-3 rounded-xl flex items-center justify-center gap-2 font-semibold shadow-lg">
            <Plus size={20} /> Catatan Baru
          </button>
        </div>
      </div>

      {/* MAIN CONTENT */}
      <div className="flex-1 flex flex-col h-full relative">
        
        {/* VIEW: TIMELINE */}
        {view === 'timeline' && (
          <div className="flex-1 overflow-y-auto p-8">
            <header className="flex justify-between items-center mb-8">
              <div>
                <h2 className="text-2xl font-bold text-white">Timeline</h2>
                <p className="text-slate-500 text-sm">Jejak aktivitasmu.</p>
              </div>
            </header>
            <div className="space-y-6 max-w-3xl">
              {logs.map((log) => (
                <div key={log.id} className="group relative pl-8 border-l border-slate-800 pb-2 last:border-0">
                  <div className="absolute -left-[5px] top-0 w-2.5 h-2.5 bg-indigo-500 rounded-full ring-4 ring-slate-950"></div>
                  <div className="bg-slate-900 border border-slate-800 rounded-2xl p-5 hover:border-slate-700 transition-all group-hover:shadow-xl">
                    <div className="flex justify-between items-start mb-3">
                      <div className="flex items-center gap-2 text-xs font-mono text-slate-500">
                        <Calendar size={12} /> {log.entry_date} 
                        <span className="mx-1">•</span> <Clock size={12} /> {log.entry_time.substring(0,5)}
                      </div>
                      <button onClick={() => deleteLog(log.id)} className="text-slate-600 hover:text-red-400 opacity-0 group-hover:opacity-100"><Trash2 size={16} /></button>
                    </div>
                    <div className="prose prose-invert prose-sm max-w-none text-slate-300 mb-4"><ReactMarkdown>{log.content}</ReactMarkdown></div>
                    <div className="flex items-center gap-2 flex-wrap">
                      <span className="px-2 py-1 bg-slate-800 rounded text-[10px] text-indigo-400 uppercase font-bold">{log.category}</span>
                      {log.tags && JSON.parse(log.tags).map((tag, i) => <span key={i} className="flex items-center gap-1 px-2 py-1 bg-slate-800/50 border border-slate-700 rounded text-[10px] text-slate-400"><Tag size={10} /> {tag}</span>)}
                      {log.mood && <span className="ml-auto flex items-center gap-1 text-xs text-slate-500"><Smile size={12} /> {log.mood}</span>}
                    </div>
                  </div>
                </div>
              ))}
              {logs.length === 0 && <div className="text-center py-20 text-slate-600"><Book size={48} className="mx-auto mb-4 opacity-20" /><p>Belum ada catatan.</p></div>}
            </div>
          </div>
        )}

        {/* VIEW: NEW ENTRY (FORM WITH PREVIEW) */}
        {view === 'new-entry' && (
          <div className="flex-1 overflow-y-auto p-8 flex justify-center">
            <div className="w-full max-w-2xl">
              <h2 className="text-2xl font-bold text-white mb-6">Catatan Baru</h2>
              
              <div className="bg-slate-900 border border-slate-800 rounded-2xl p-6 space-y-6 shadow-xl">
                
                {/* Waktu & Tanggal */}
                <div className="grid grid-cols-2 gap-4">
                  <div>
                    <label className="block text-xs font-medium text-slate-500 mb-2">TANGGAL</label>
                    <input type="date" className="w-full bg-slate-950 border border-slate-800 rounded-lg p-3 text-white focus:border-indigo-500 outline-none" value={newEntry.entry_date} onChange={e => setNewEntry({...newEntry, entry_date: e.target.value})} />
                  </div>
                  <div>
                    <label className="block text-xs font-medium text-slate-500 mb-2">WAKTU</label>
                    <input type="time" className="w-full bg-slate-950 border border-slate-800 rounded-lg p-3 text-white focus:border-indigo-500 outline-none" value={newEntry.entry_time} onChange={e => setNewEntry({...newEntry, entry_time: e.target.value})} />
                  </div>
                </div>

                {/* EDITOR AREA (WRITE / PREVIEW) */}
                <div>
                  <div className="flex justify-between items-end mb-2 border-b border-slate-800 pb-2">
                    {/* TABS */}
                    <div className="flex gap-4">
                      <button 
                        onClick={() => setEditorMode('write')}
                        className={`flex items-center gap-2 pb-2 text-sm font-medium transition-colors ${editorMode === 'write' ? 'text-indigo-400 border-b-2 border-indigo-400 -mb-2.5' : 'text-slate-500 hover:text-slate-300'}`}
                      >
                        <Edit2 size={14} /> Write
                      </button>
                      <button 
                        onClick={() => setEditorMode('preview')}
                        className={`flex items-center gap-2 pb-2 text-sm font-medium transition-colors ${editorMode === 'preview' ? 'text-indigo-400 border-b-2 border-indigo-400 -mb-2.5' : 'text-slate-500 hover:text-slate-300'}`}
                      >
                        <Eye size={14} /> Preview
                      </button>
                    </div>

                    {/* AI BUTTON (Only show in Write mode) */}
                    {editorMode === 'write' && (
                      <button 
                        onClick={async () => {
                          if (!newEntry.content) return;
                          setLoading(true);
                          try {
                            const res = await fetch('http://localhost:3000/api/ai/polish', { method: 'POST', headers: { 'Content-Type': 'application/json' }, body: JSON.stringify({ draft_content: newEntry.content }) });
                            const data = await res.json();
                            setNewEntry(prev => ({ ...prev, content: data.polished_content }));
                          } catch(e) { alert("AI Error") } finally { setLoading(false); }
                        }}
                        disabled={loading || !newEntry.content}
                        className="flex items-center gap-1 text-xs bg-indigo-900/50 text-indigo-300 px-3 py-1 rounded-full hover:bg-indigo-600 hover:text-white transition-all disabled:opacity-50"
                      >
                        {loading ? "Polishing..." : "✨ AI Polish"}
                      </button>
                    )}
                  </div>

                  {/* KONTEN EDITOR */}
                  <div className="relative min-h-[200px]">
                    {editorMode === 'write' ? (
                      <textarea 
                        className="w-full h-64 bg-slate-950 border border-slate-800 rounded-lg p-4 text-white focus:border-indigo-500 outline-none resize-none font-mono text-sm leading-relaxed"
                        placeholder="Tulis logbook di sini (Markdown supported)..."
                        value={newEntry.content}
                        onChange={e => setNewEntry({...newEntry, content: e.target.value})}
                      ></textarea>
                    ) : (
                      <div className="w-full h-64 bg-slate-900/50 border border-slate-800 rounded-lg p-4 overflow-y-auto prose prose-invert prose-sm max-w-none">
                        {newEntry.content ? (
                          <ReactMarkdown>{newEntry.content}</ReactMarkdown>
                        ) : (
                          <p className="text-slate-600 italic">Belum ada konten untuk dipratinjau.</p>
                        )}
                      </div>
                    )}
                  </div>
                </div>

                {/* Metadata Lainnya */}
                <div className="grid grid-cols-3 gap-4">
                  <div>
                    <label className="block text-xs font-medium text-slate-500 mb-2">KATEGORI</label>
                    <select className="w-full bg-slate-950 border border-slate-800 rounded-lg p-3 text-white outline-none" value={newEntry.category} onChange={e => setNewEntry({...newEntry, category: e.target.value})}>
                      <option>General</option>
                      <option>Work</option>
                      <option>Personal</option>
                      <option>Project Noty</option>
                    </select>
                  </div>
                  <div>
                    <label className="block text-xs font-medium text-slate-500 mb-2">MOOD</label>
                    <select className="w-full bg-slate-950 border border-slate-800 rounded-lg p-3 text-white outline-none" value={newEntry.mood} onChange={e => setNewEntry({...newEntry, mood: e.target.value})}>
                      <option>Neutral</option>
                      <option>Productive</option>
                      <option>Tired</option>
                      <option>Excited</option>
                      <option>Stressed</option>
                    </select>
                  </div>
                  <div>
                    <label className="block text-xs font-medium text-slate-500 mb-2">TAGS</label>
                    <input type="text" className="w-full bg-slate-950 border border-slate-800 rounded-lg p-3 text-white outline-none focus:border-indigo-500" placeholder="tag1, tag2" value={newEntry.tags} onChange={e => setNewEntry({...newEntry, tags: e.target.value})} />
                  </div>
                </div>

                <div className="pt-4 flex gap-3">
                  <button onClick={() => setView('timeline')} className="flex-1 py-3 text-slate-400 hover:text-white transition-colors">Batal</button>
                  <button onClick={submitLog} disabled={loading} className="flex-1 bg-emerald-600 hover:bg-emerald-700 text-white font-bold py-3 rounded-xl shadow-lg transition-all">{loading ? 'Menyimpan...' : 'Simpan Log'}</button>
                </div>
              </div>
            </div>
          </div>
        )}

        {/* VIEW: PERSONAS */}
        {view === 'personas' && (
          <div className="flex-1 overflow-y-auto p-8">
            <h2 className="text-2xl font-bold text-white mb-6">AI Personas</h2>
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
              {personas.map(p => (
                <div key={p.id} className={`p-6 rounded-2xl border transition-all ${p.is_active ? 'bg-indigo-900/20 border-indigo-500' : 'bg-slate-900 border-slate-800'}`}>
                  <div className="flex justify-between items-start mb-4">
                    <div className={`p-3 rounded-lg ${p.is_active ? 'bg-indigo-600 text-white' : 'bg-slate-800 text-slate-400'}`}><Brain size={24} /></div>
                    {p.is_active && <span className="text-[10px] bg-indigo-500 text-white px-2 py-1 rounded-full font-bold">ACTIVE</span>}
                  </div>
                  <h3 className="font-bold text-lg text-white mb-2">{p.name}</h3>
                  <p className="text-slate-400 text-sm mb-6 h-10">{p.description}</p>
                  <button onClick={() => activatePersona(p.id)} disabled={p.is_active} className={`w-full py-2 rounded-lg font-medium text-sm ${p.is_active ? 'bg-slate-800 text-slate-500 cursor-default' : 'bg-white text-slate-900 hover:bg-slate-200'}`}>{p.is_active ? 'Sedang Digunakan' : 'Aktifkan'}</button>
                </div>
              ))}
            </div>
          </div>
        )}

        {/* VIEW: SETTINGS (REAL) */}
        {view === 'settings' && (
          <div className="flex-1 overflow-y-auto p-8 flex justify-center">
            <div className="w-full max-w-xl">
              <h2 className="text-2xl font-bold text-white mb-6">Settings</h2>
              <div className="bg-slate-900 border border-slate-800 rounded-2xl p-6 space-y-6">
                
                <div className="space-y-2">
                  <label className="text-xs font-medium text-slate-500">USERNAME</label>
                  <input type="text" className="w-full bg-slate-950 border border-slate-800 rounded-lg p-3 text-white outline-none focus:border-indigo-500" 
                    value={settingsConfig.username} onChange={e => setSettingsConfig({...settingsConfig, username: e.target.value})} 
                  />
                </div>

               <div className="space-y-2">
                  <label className="text-xs font-medium text-slate-500">AI MODEL</label>
                  <select className="w-full bg-slate-950 border border-slate-800 rounded-lg p-3 text-white outline-none focus:border-indigo-500"
                    value={settingsConfig.ai_model_name} onChange={e => setSettingsConfig({...settingsConfig, ai_model_name: e.target.value})}
                  >
                    {/* MODEL TERBARU (GEMINI 3) */}
                    <option value="gemini-3-flash-preview">Gemini 3 Flash Preview (Tercepat & Cerdas)</option>
                    <option value="gemini-3-pro-preview">Gemini 3 Pro Preview (Reasoning Tinggi)</option>
                    
                    {/* MODEL STABIL (GEMINI 2.5) */}
                    <option value="gemini-2.5-flash">Gemini 2.5 Flash (Stabil & Hemat)</option>
                    <option value="gemini-2.5-pro">Gemini 2.5 Pro (Stabil & Powerful)</option>
                    
                    {/* HEMAT BIAYA */}
                    <option value="gemini-2.5-flash-lite">Gemini 2.5 Flash Lite (Sangat Hemat)</option>
                  </select>
                </div>

                <div className="pt-4 border-t border-slate-800 space-y-2">
                   <div className="flex justify-between">
                     <label className="text-xs font-medium text-slate-500">GEMINI API KEY</label>
                     {settingsConfig.is_key_set && <span className="text-xs text-green-400 flex items-center gap-1"><CheckCircle size={12} /> Key Tersimpan</span>}
                   </div>
                   <input type="password" autoComplete="new-password" className="w-full bg-slate-950 border border-slate-800 rounded-lg p-3 text-white outline-none focus:border-indigo-500 placeholder-slate-700" 
                     placeholder={settingsConfig.is_key_set ? "Biarkan kosong jika tidak ingin mengubah" : "Tempel API Key di sini (Wajib)"}
                     value={settingsConfig.ai_api_key} onChange={e => setSettingsConfig({...settingsConfig, ai_api_key: e.target.value})}
                   />
                </div>

                <button onClick={saveSettings} disabled={loading} className="w-full bg-indigo-600 hover:bg-indigo-700 text-white font-bold py-3 rounded-xl shadow-lg mt-4 flex justify-center gap-2">
                   {loading ? "Menyimpan..." : <><Save size={20} /> Simpan Pengaturan</>}
                </button>

                {/* --- SEPARATOR --- */}
                <div className="border-t border-slate-800 pt-6">
                  <div className="flex items-center gap-2 mb-4">
                    <Github size={20} className="text-white" />
                    <h3 className="font-bold text-lg text-white">Integrasi GitHub</h3>
                  </div>
                  
                  <div className="bg-slate-950/50 rounded-xl p-4 border border-slate-800 space-y-4">
                    <div className="flex justify-between items-center">
                      <label className="text-xs font-medium text-slate-500">STATUS INTEGRASI</label>
                      <button 
                        onClick={() => setGithubConfig({...githubConfig, is_active: !githubConfig.is_active})}
                        className={`transition-colors ${githubConfig.is_active ? 'text-emerald-400' : 'text-slate-600'}`}
                      >
                        {githubConfig.is_active ? <ToggleRight size={32} /> : <ToggleLeft size={32} />}
                      </button>
                    </div>

                    <div className="space-y-2">
                      <label className="text-xs font-medium text-slate-500">REPOSITORY (Owner/Repo)</label>
                      <input type="text" className="w-full bg-slate-900 border border-slate-800 rounded-lg p-3 text-white outline-none focus:border-indigo-500 font-mono text-sm" 
                        placeholder="contoh: clownface471/Noty"
                        value={githubConfig.repo_name} onChange={e => setGithubConfig({...githubConfig, repo_name: e.target.value})} 
                      />
                    </div>

                    <div className="space-y-2">
                      <div className="flex justify-between">
                         <label className="text-xs font-medium text-slate-500">PERSONAL ACCESS TOKEN</label>
                         {githubConfig.is_token_set && <span className="text-xs text-green-400 flex items-center gap-1"><CheckCircle size={12} /> Terhubung</span>}
                      </div>
                      <input type="password" autoComplete="new-password" className="w-full bg-slate-900 border border-slate-800 rounded-lg p-3 text-white outline-none focus:border-indigo-500 placeholder-slate-700" 
                        placeholder={githubConfig.is_token_set ? "Token tersimpan (biarkan kosong)" : "Wajib untuk repo private"}
                        value={githubConfig.token} onChange={e => setGithubConfig({...githubConfig, token: e.target.value})}
                      />
                      <p className="text-[10px] text-slate-600">
                        *Noty akan mengecek commit baru setiap 60 detik dan mencatatnya otomatis.
                      </p>
                    </div>

                    <button onClick={saveGithub} disabled={loading} className="w-full bg-slate-800 hover:bg-slate-700 text-white font-medium py-3 rounded-xl transition-all mt-2">
                      Simpan Integrasi GitHub
                    </button>
                  </div>
                </div>
                
                {msg && <div className={`text-center text-sm p-2 rounded ${msg.includes('Gagal') ? 'text-red-400 bg-red-900/20' : 'text-green-400 bg-green-900/20'}`}>{msg}</div>}
              </div>
            </div>
          </div>
        )}

      </div>
    </div>
  )
}

export default App