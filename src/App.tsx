import { FormEvent, useEffect, useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import "./App.css";

type ChatEntry = {
  from: string;
  message: string;
};

function App() {
  const [serverAddress, setServerAddress] = useState("Loading...");
  const [connectAddress, setConnectAddress] = useState("127.0.0.1:8000");
  const [connections, setConnections] = useState<string[]>([]);
  const [selectedConnection, setSelectedConnection] = useState("");
  const [outboundMessage, setOutboundMessage] = useState("");
  const [messagesByConnection, setMessagesByConnection] = useState<
    Record<string, ChatEntry[]>
  >({});
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
      if (!selectedConnection && list.length > 0) {
        setSelectedConnection(list[0]);
      }
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
    if (!connectionId) {
      return;
    }

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
    if (!addr) {
      setStatus("Enter a host address to connect.");
      return;
    }
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
    if (!canSend) {
      return;
    }

    const text = outboundMessage.trim();
    setOutboundMessage("");
    try {
      await invoke<void>("send_simple_text", {
        connId: selectedConnection,
        msg: text,
      });
      setMessagesByConnection((current) => ({
        ...current,
        [selectedConnection]: [
          ...(current[selectedConnection] ?? []),
          { from: "You", message: text },
        ],
      }));
      setStatus("Message sent");
    } catch (error) {
      setStatus(`Send failed: ${String(error)}`);
    }
  };

  useEffect(() => {
    void loadServerAddress();
    void refreshConnections();
  }, []);

  const selectedConnectionLabel = selectedConnection || "No connection selected";
  const visibleMessages = selectedConnection
    ? messagesByConnection[selectedConnection] ?? []
    : [];

  return (
    <main className="mx-auto flex min-h-screen w-full max-w-6xl flex-col gap-4 p-4 text-slate-100 md:p-8">
      <header className="rounded-2xl border border-slate-700/60 bg-slate-900/70 p-5 backdrop-blur">
        <div className="flex flex-col gap-2 md:flex-row md:items-end md:justify-between">
          <div>
            <p className="text-xs uppercase tracking-[0.2em] text-cyan-300">Ghostline</p>
            <h1 className="text-3xl font-semibold">Local Chat Console</h1>
          </div>
          <div className="rounded-lg border border-cyan-400/40 bg-cyan-500/10 px-3 py-2 text-sm">
            <p className="text-cyan-200">Server</p>
            <p className="font-mono text-cyan-100">{serverAddress}</p>
          </div>
        </div>
      </header>

      <section className="grid flex-1 gap-4 md:grid-cols-[320px_1fr]">
        <aside className="space-y-4 rounded-2xl border border-slate-700/60 bg-slate-900/65 p-4 backdrop-blur">
          <h2 className="text-sm font-medium uppercase tracking-wider text-slate-300">
            Connections
          </h2>
          <form className="space-y-2" onSubmit={handleConnect}>
            <label className="block text-xs text-slate-400" htmlFor="connect-address">
              Host Address
            </label>
            <input
              id="connect-address"
              value={connectAddress}
              onChange={(event) => setConnectAddress(event.currentTarget.value)}
              className="w-full rounded-lg border border-slate-600 bg-slate-950/70 px-3 py-2 text-sm text-slate-100 outline-none transition focus:border-cyan-400"
              placeholder="127.0.0.1:8000"
            />
            <button
              type="submit"
              className="w-full rounded-lg bg-cyan-500 px-3 py-2 text-sm font-medium text-slate-950 transition hover:bg-cyan-400"
            >
              Connect
            </button>
          </form>
          <button
            type="button"
            onClick={() => void refreshConnections()}
            className="w-full rounded-lg border border-slate-600 px-3 py-2 text-sm text-slate-200 transition hover:border-cyan-400"
          >
            Refresh List
          </button>
          <div className="max-h-60 space-y-2 overflow-auto pr-1">
            {connections.length === 0 && (
              <p className="text-sm text-slate-400">No active connections yet.</p>
            )}
            {connections.map((connectionId) => (
              <div key={connectionId} className="flex items-center gap-2">
                <button
                  type="button"
                  onClick={() => setSelectedConnection(connectionId)}
                  className={`flex-1 rounded-lg border px-3 py-2 text-left text-sm transition ${
                    connectionId === selectedConnection
                      ? "border-cyan-400 bg-cyan-500/20 text-cyan-100"
                      : "border-slate-700 bg-slate-950/40 text-slate-300 hover:border-slate-500"
                  }`}
                >
                  <span className="font-mono text-xs">{connectionId}</span>
                </button>
                <button
                  type="button"
                  onClick={() => void refreshChat(connectionId)}
                  disabled={refreshingConnection === connectionId}
                  className="rounded-lg border border-slate-600 px-3 py-2 text-xs text-slate-200 transition hover:border-cyan-400 disabled:cursor-not-allowed disabled:border-slate-700 disabled:text-slate-500"
                >
                  {refreshingConnection === connectionId ? "..." : "Refresh"}
                </button>
              </div>
            ))}
          </div>
        </aside>

        <section className="flex flex-col rounded-2xl border border-slate-700/60 bg-slate-900/65 p-4 backdrop-blur">
          <div className="mb-4 flex items-center justify-between gap-2 border-b border-slate-700 pb-3">
            <div>
              <h2 className="text-xl font-medium">Conversation</h2>
              <p className="font-mono text-xs text-slate-400">{selectedConnectionLabel}</p>
            </div>
            <div className="flex items-center gap-2">
              {selectedConnection && (
                <button
                  type="button"
                  onClick={() => void refreshChat(selectedConnection)}
                  disabled={refreshingConnection === selectedConnection}
                  className="rounded-lg border border-slate-600 px-3 py-2 text-xs text-slate-200 transition hover:border-cyan-400 disabled:cursor-not-allowed disabled:border-slate-700 disabled:text-slate-500"
                >
                  {refreshingConnection === selectedConnection ? "Refreshing..." : "Refresh Chat"}
                </button>
              )}
              <p className="text-xs text-slate-400">{status}</p>
            </div>
          </div>

          <div className="mb-4 flex-1 space-y-3 overflow-auto rounded-xl bg-slate-950/55 p-3">
            {visibleMessages.length === 0 && (
              <p className="text-sm text-slate-500">
                No messages yet. Connect and send a message to begin.
              </p>
            )}
            {visibleMessages.map((entry, index) => (
              <article
                key={`${entry.from}-${entry.message}-${index}`}
                className="max-w-[85%] rounded-lg border border-cyan-500/20 bg-cyan-500/10 px-3 py-2"
              >
                <p className="text-xs text-cyan-200">{entry.from}</p>
                <p className="text-sm text-slate-100">{entry.message}</p>
              </article>
            ))}
          </div>

          <form className="flex gap-2" onSubmit={handleSend}>
            <input
              value={outboundMessage}
              onChange={(event) => setOutboundMessage(event.currentTarget.value)}
              className="flex-1 rounded-lg border border-slate-600 bg-slate-950/80 px-3 py-2 text-sm text-slate-100 outline-none transition focus:border-cyan-400"
              placeholder="Type a message..."
            />
            <button
              type="submit"
              disabled={!canSend}
              className="rounded-lg bg-emerald-500 px-4 py-2 text-sm font-medium text-slate-950 transition hover:bg-emerald-400 disabled:cursor-not-allowed disabled:bg-slate-700 disabled:text-slate-400"
            >
              Send
            </button>
          </form>
        </section>
      </section>
    </main>
  );
  }

export default App;
