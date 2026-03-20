import { FormEvent, useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import "./App.css";

type ChatEntry = {
  from: string;
  message: string;
};

type MessageEventPayload = {
  connection_id: string;
  from: string;
  message: string;
};

type ConnectionEventPayload = {
  connection_id: string;
};

function App() {
  const [serverAddress, setServerAddress] = useState("Loading...");
  const [connectAddress, setConnectAddress] = useState("127.0.0.1:8000");
  const [connections, setConnections] = useState<string[]>([]);
  const [selectedConnection, setSelectedConnection] = useState("");
  const [outboundMessage, setOutboundMessage] = useState("");
  const [messagesByConnection, setMessagesByConnection] = useState<Record<string, ChatEntry[]>>({});
  const [refreshingConnection, setRefreshingConnection] = useState("");
  const [status, setStatus] = useState("Idle");

  const canSend = useMemo(
    () => selectedConnection.length > 0 && outboundMessage.trim().length > 0,
    [selectedConnection, outboundMessage],
  );

  const refreshConnections = async () => {
    try {
      const list = await invoke<string[]>("get_my_connections");
      setConnections(list);
      if (!selectedConnection && list.length > 0) setSelectedConnection(list[0]);
    } catch (error) {
      setStatus(`Failed to load connections: ${String(error)}`);
    }
  };

  const loadServerAddress = async () => {
    try {
      const addr = await invoke<string>("get_server_address");
      setServerAddress(addr);
    } catch (error) {
      setServerAddress(`Error: ${String(error)}`);
    }
  };

  const refreshChat = async (connectionId: string) => {
    if (!connectionId) return;
    setRefreshingConnection(connectionId);
    setStatus(`Refreshing ${connectionId}...`);
    try {
      const history = await invoke<[string, string][]>("get_connection_messages", {
        id: connectionId,
        limit: 100,
        skip: 0,
      });
      setMessagesByConnection((current) => ({
        ...current,
        [connectionId]: history.map(([from, message]) => ({ from, message })),
      }));
      setStatus(`Refreshed ${connectionId}`);
    } catch (error) {
      setStatus(`Refresh failed: ${String(error)}`);
    } finally {
      setRefreshingConnection("");
    }
  };

  const handleConnect = async (event: FormEvent) => {
    event.preventDefault();
    const addr = connectAddress.trim();
    if (!addr) { setStatus("Enter a host address to connect."); return; }
    setStatus(`Connecting to ${addr}...`);
    try {
      await invoke<boolean>("connect_to_host", { addr });
      await refreshConnections();
      setSelectedConnection(addr);
      await refreshChat(addr);
      setStatus(`Connected to ${addr}`);
    } catch (error) {
      setStatus(`Connection failed: ${String(error)}`);
    }
  };

  const handleSend = async (event: FormEvent) => {
    event.preventDefault();
    if (!canSend) return;
    const text = outboundMessage.trim();
    setOutboundMessage("");
    try {
      await invoke<void>("send_simple_text", { connId: selectedConnection, msg: text });
      await refreshChat(selectedConnection);
      setStatus("Message sent and synced");
    } catch (error) {
      setStatus(`Send failed: ${String(error)}`);
    }
  };

  useEffect(() => {
    void loadServerAddress();
    void refreshConnections();
  }, []);

  useEffect(() => {
    if (!selectedConnection) return;
    void refreshChat(selectedConnection);
  }, [selectedConnection]);

  useEffect(() => {
    let isMounted = true;
    let unlistenMessage: (() => void) | undefined;
    let unlistenConnection: (() => void) | undefined;

    void listen<MessageEventPayload>("ghostline://message-received", (event) => {
      if (!isMounted) return;

      const { connection_id, from, message } = event.payload;
      setMessagesByConnection((current) => ({
        ...current,
        [connection_id]: [...(current[connection_id] ?? []), { from, message }],
      }));

      if (selectedConnection === connection_id) {
        setStatus(`Live update from ${connection_id}`);
      }
    }).then((dispose) => {
      unlistenMessage = dispose;
    });

    void listen<ConnectionEventPayload>("ghostline://connection-created", (event) => {
      if (!isMounted) return;

      const { connection_id } = event.payload;
      setConnections((current) =>
        current.includes(connection_id) ? current : [...current, connection_id],
      );
      setStatus(`New connection ${connection_id}`);
    }).then((dispose) => {
      unlistenConnection = dispose;
    });

    return () => {
      isMounted = false;
      if (unlistenMessage) void unlistenMessage();
      if (unlistenConnection) void unlistenConnection();
    };
  }, [selectedConnection]);

  const selectedConnectionLabel = selectedConnection || "No connection selected";
  const visibleMessages = selectedConnection ? messagesByConnection[selectedConnection] ?? [] : [];

  return (
    <main className="h-screen overflow-hidden bg-[#11111b] text-[#cdd6f4] font-sans">
      <section className="mx-auto grid h-screen w-full max-w-7xl overflow-hidden md:grid-cols-[260px_1fr]">

        {/* Sidebar */}
        <aside className="flex min-h-0 flex-col border-b border-[#1e1e2e] bg-[#181825] md:border-b-0 md:border-r md:border-[#1e1e2e]">

          {/* Branding */}
          <div className="px-4 py-5 border-b border-[#1e1e2e]">
            <p className="text-[10px] uppercase tracking-[0.22em] text-[#6c7086]">Ghostline</p>
            <h1 className="mt-1 text-[15px] font-medium text-[#f5e0dc]">Chats</h1>
            <p className="mt-2 font-mono text-[11px] text-[#45475a]">{serverAddress}</p>
          </div>

          {/* Connect form */}
          <form className="flex gap-2 px-4 py-3 border-b border-[#1e1e2e]" onSubmit={handleConnect}>
            <input
              id="connect-address"
              value={connectAddress}
              onChange={(e) => setConnectAddress(e.currentTarget.value)}
              className="flex-1 min-w-0 bg-[#11111b] border border-[#313244] rounded px-2.5 py-1.5 font-mono text-xs text-[#cdd6f4] placeholder:text-[#45475a] outline-none focus:border-[#585b70]"
              placeholder="host:port"
            />
            <button
              type="submit"
              className="bg-[#89b4fa] rounded px-3 py-1.5 text-xs font-medium text-[#11111b] hover:bg-[#b4befe] transition-colors"
            >
              Connect
            </button>
          </form>

          {/* Connection list */}
          <div className="flex-1 overflow-y-auto px-3 py-2">
            {connections.length === 0 ? (
              <p className="px-2 py-2 text-xs text-[#45475a]">No active connections.</p>
            ) : (
              connections.map((id) => (
                <div
                  key={id}
                  onClick={() => setSelectedConnection(id)}
                  className={`flex items-center gap-2 px-2.5 py-2 rounded-md cursor-pointer mb-0.5 transition-colors ${
                    id === selectedConnection ? "bg-[#1e1e2e]" : "hover:bg-[#1e1e2e]/60"
                  }`}
                >
                  <span className={`w-1.5 h-1.5 rounded-full shrink-0 ${
                    id === selectedConnection ? "bg-[#a6e3a1]" : "bg-[#585b70]"
                  }`} />
                  <span className="flex-1 font-mono text-xs text-[#bac2de] truncate">{id}</span>
                </div>
              ))
            )}
          </div>
        </aside>

        {/* Main chat area */}
        <section className="flex min-h-0 flex-col overflow-hidden bg-[#11111b]">

          {/* Header */}
          <header className="flex items-center justify-between border-b border-[#1e1e2e] px-5 py-3.5 shrink-0">
            <p className="font-mono text-[13px] text-[#f5e0dc]">{selectedConnectionLabel}</p>
            <div className="flex items-center gap-3">
              {selectedConnection && (
                <button
                  type="button"
                  onClick={() => void refreshChat(selectedConnection)}
                  disabled={refreshingConnection === selectedConnection}
                  className="text-[10px] font-mono uppercase tracking-[0.16em] text-[#585b70] transition-colors hover:text-[#89b4fa] disabled:text-[#313244]"
                >
                  {refreshingConnection === selectedConnection ? "sync" : "refresh"}
                </button>
              )}
              <p className="text-[11px] text-[#45475a]">{status}</p>
            </div>
          </header>

          {/* Messages */}
          <div className="min-h-0 flex-1 overflow-y-auto px-5 py-5">
            <div className="mx-auto w-full max-w-3xl flex flex-col gap-5">
              {visibleMessages.length === 0 ? (
                <p className="text-xs text-[#45475a] leading-relaxed">
                  No messages yet. Select a connection, refresh, or send a message.
                </p>
              ) : (
                visibleMessages.map((entry, index) => {
                  const isMe = entry.from === "You";
                  return (
                    <article key={`${entry.from}-${index}`} className="flex gap-3">
                      <span className={`mt-0.5 w-0.5 shrink-0 rounded-sm self-stretch ${
                        isMe ? "bg-[#89b4fa]" : "bg-[#fab387]"
                      }`} />
                      <div className="min-w-0">
                        <p className={`mb-1 text-[10px] uppercase tracking-[0.16em] ${
                          isMe ? "text-[#89b4fa]" : "text-[#fab387]"
                        }`}>
                          {entry.from}
                        </p>
                        <p className="text-[13px] leading-7 text-[#bac2de] whitespace-pre-wrap break-words">
                          {entry.message}
                        </p>
                      </div>
                    </article>
                  );
                })
              )}
            </div>
          </div>

          {/* Compose */}
          <footer className="border-t border-[#1e1e2e] px-5 py-3 shrink-0">
            <form className="mx-auto flex w-full max-w-3xl gap-2" onSubmit={handleSend}>
              <input
                value={outboundMessage}
                onChange={(e) => setOutboundMessage(e.currentTarget.value)}
                className="flex-1 bg-[#181825] border border-[#313244] rounded-md px-3 py-2.5 text-[13px] text-[#cdd6f4] placeholder:text-[#45475a] outline-none focus:border-[#585b70]"
                placeholder="Write a message"
              />
              <button
                type="submit"
                disabled={!canSend}
                className="bg-[#a6e3a1] rounded-md px-4 py-2.5 text-[13px] font-medium text-[#11111b] hover:bg-[#94e2d5] disabled:bg-[#313244] disabled:text-[#45475a] transition-colors"
              >
                Send
              </button>
            </form>
          </footer>

        </section>
      </section>
    </main>
  );
}

export default App;
