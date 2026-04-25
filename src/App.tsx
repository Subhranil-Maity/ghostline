import { FormEvent, useEffect, useMemo, useRef, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import ChatComposer from "./components/ChatComposer";
import ChatHeader from "./components/ChatHeader";
import MessageList from "./components/MessageList";
import Sidebar from "./components/Sidebar";
import {
  ConnectionEventPayload,
  HistoryEntry,
  MessageEventPayload,
} from "./types/chat";
import "./App.css";

function App() {
  const [serverAddress, setServerAddress] = useState("Loading...");
  const [connectAddress, setConnectAddress] = useState("127.0.0.1:8000");
  const [connections, setConnections] = useState<string[]>([]);
  const [selectedConnection, setSelectedConnection] = useState("");
  const [outboundMessage, setOutboundMessage] = useState("");
  const [messagesByConnection, setMessagesByConnection] = useState<Record<string, HistoryEntry[]>>({});
  const [status, setStatus] = useState("Idle");
  const messageScrollRef = useRef<HTMLDivElement | null>(null);
  const shouldStickToBottomRef = useRef(true);

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
    setStatus(`Refreshing ${connectionId}...`);
    try {
      const history = await invoke<HistoryEntry[]>("get_connection_messages", {
        id: connectionId,
        limit: 100,
        skip: 0,
      });
      setMessagesByConnection((current) => ({
        ...current,
        [connectionId]: history,
      }));
      setStatus(`Refreshed ${connectionId}`);
    } catch (error) {
      setStatus(`Refresh failed: ${String(error)}`);
    }
  };

  const handleConnect = async (event: FormEvent) => {
    event.preventDefault();
    const addr = connectAddress.trim();
    if (!addr) { setStatus("Enter a host address to connect."); return; }
    setStatus(`Connecting to ${addr}...`);
    try {
      await invoke<boolean>("connect_to_host", { addr });
      setStatus(`Connected to ${addr}, waiting for peer handshake...`);
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
    shouldStickToBottomRef.current = true;
    void refreshChat(selectedConnection);
  }, [selectedConnection]);

  useEffect(() => {
    let isMounted = true;
    let unlistenMessage: (() => void) | undefined;
    let unlistenConnection: (() => void) | undefined;

    void listen<MessageEventPayload>("ghostline://message-received", (event) => {
      if (!isMounted) return;

      const { peer_id, message } = event.payload;
      setMessagesByConnection((current) => ({
        ...current,
        [peer_id]: [...(current[peer_id] ?? []), message],
      }));

      if (selectedConnection === peer_id) {
        setStatus(`Live update from ${peer_id}`);
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
      setSelectedConnection((current) => current || connection_id);
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

  useEffect(() => {
    const container = messageScrollRef.current;
    if (!container) return;
    if (!shouldStickToBottomRef.current) return;

    const frame = window.requestAnimationFrame(() => {
      container.scrollTop = container.scrollHeight;
    });

    return () => window.cancelAnimationFrame(frame);
  }, [selectedConnection, visibleMessages.length]);

  const handleMessageScroll = () => {
    const container = messageScrollRef.current;
    if (!container) return;

    const distanceFromBottom =
      container.scrollHeight - container.scrollTop - container.clientHeight;
    shouldStickToBottomRef.current = distanceFromBottom < 48;
  };

  return (
    <main className="h-screen overflow-hidden bg-[#11111b] text-[#cdd6f4] font-sans">
      <section className="mx-auto grid h-screen w-full max-w-7xl overflow-hidden md:grid-cols-[260px_1fr]">
        <Sidebar
          serverAddress={serverAddress}
          connectAddress={connectAddress}
          connections={connections}
          selectedConnection={selectedConnection}
          onConnectAddressChange={setConnectAddress}
          onConnect={handleConnect}
          onSelectConnection={setSelectedConnection}
        />

        <section className="flex min-h-0 flex-col overflow-hidden bg-[#11111b]">
          <ChatHeader
            selectedConnectionLabel={selectedConnectionLabel}
            status={status}
          />
          <MessageList
            messages={visibleMessages}
            scrollRef={messageScrollRef}
            onScroll={handleMessageScroll}
          />
          <ChatComposer
            outboundMessage={outboundMessage}
            canSend={canSend}
            onMessageChange={setOutboundMessage}
            onSend={handleSend}
          />
        </section>
      </section>
    </main>
  );
}

export default App;
