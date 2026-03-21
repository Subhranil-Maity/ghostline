type ChatHeaderProps = {
  selectedConnectionLabel: string;
  status: string;
};

function ChatHeader({
  selectedConnectionLabel,
  status,
}: ChatHeaderProps) {
  return (
    <header className="flex items-center justify-between border-b border-[#1e1e2e] px-5 py-3.5 shrink-0">
      <p className="font-mono text-[13px] text-[#f5e0dc]">{selectedConnectionLabel}</p>
      <p className="text-[11px] text-[#45475a]">{status}</p>
    </header>
  );
}

export default ChatHeader;
