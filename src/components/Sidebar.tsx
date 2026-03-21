import { FormEvent } from "react";

type SidebarProps = {
  serverAddress: string;
  connectAddress: string;
  connections: string[];
  selectedConnection: string;
  onConnectAddressChange: (value: string) => void;
  onConnect: (event: FormEvent) => void;
  onSelectConnection: (connectionId: string) => void;
};

function Sidebar({
  serverAddress,
  connectAddress,
  connections,
  selectedConnection,
  onConnectAddressChange,
  onConnect,
  onSelectConnection,
}: SidebarProps) {
  return (
    <aside className="flex min-h-0 flex-col border-b border-[#1e1e2e] bg-[#181825] md:border-b-0 md:border-r md:border-[#1e1e2e]">
      <div className="border-b border-[#1e1e2e] px-4 py-5">
        <p className="text-[10px] uppercase tracking-[0.22em] text-[#6c7086]">Ghostline</p>
        <h1 className="mt-1 text-[15px] font-medium text-[#f5e0dc]">Chats</h1>
        <p className="mt-2 font-mono text-[11px] text-[#45475a]">{serverAddress}</p>
      </div>

      <form className="flex gap-2 border-b border-[#1e1e2e] px-4 py-3" onSubmit={onConnect}>
        <input
          id="connect-address"
          value={connectAddress}
          onChange={(event) => onConnectAddressChange(event.currentTarget.value)}
          className="flex-1 min-w-0 rounded border border-[#313244] bg-[#11111b] px-2.5 py-1.5 font-mono text-xs text-[#cdd6f4] placeholder:text-[#45475a] outline-none focus:border-[#585b70]"
          placeholder="host:port"
        />
        <button
          type="submit"
          className="rounded bg-[#89b4fa] px-3 py-1.5 text-xs font-medium text-[#11111b] transition-colors hover:bg-[#b4befe]"
        >
          Connect
        </button>
      </form>

      <div className="themed-scrollbar flex-1 overflow-y-auto px-3 py-2">
        {connections.length === 0 ? (
          <p className="px-2 py-2 text-xs text-[#45475a]">No active connections.</p>
        ) : (
          connections.map((connectionId) => (
            <div
              key={connectionId}
              onClick={() => onSelectConnection(connectionId)}
              className={`mb-0.5 flex cursor-pointer items-center gap-2 rounded-md px-2.5 py-2 transition-colors ${
                connectionId === selectedConnection ? "bg-[#1e1e2e]" : "hover:bg-[#1e1e2e]/60"
              }`}
            >
              <span
                className={`h-1.5 w-1.5 shrink-0 rounded-full ${
                  connectionId === selectedConnection ? "bg-[#a6e3a1]" : "bg-[#585b70]"
                }`}
              />
              <span className="flex-1 truncate font-mono text-xs text-[#bac2de]">
                {connectionId}
              </span>
            </div>
          ))
        )}
      </div>
    </aside>
  );
}

export default Sidebar;
