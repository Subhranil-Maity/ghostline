import { FormEvent } from "react";

type ChatComposerProps = {
  outboundMessage: string;
  canSend: boolean;
  onMessageChange: (value: string) => void;
  onSend: (event: FormEvent) => void;
};

function ChatComposer({
  outboundMessage,
  canSend,
  onMessageChange,
  onSend,
}: ChatComposerProps) {
  return (
    <footer className="border-t border-[#1e1e2e] px-5 py-3 shrink-0">
      <form className="mx-auto flex w-full max-w-3xl gap-2" onSubmit={onSend}>
        <input
          value={outboundMessage}
          onChange={(event) => onMessageChange(event.currentTarget.value)}
          className="flex-1 rounded-md border border-[#313244] bg-[#181825] px-3 py-2.5 text-[13px] text-[#cdd6f4] placeholder:text-[#45475a] outline-none focus:border-[#585b70]"
          placeholder="Write a message"
        />
        <button
          type="submit"
          disabled={!canSend}
          className="rounded-md bg-[#a6e3a1] px-4 py-2.5 text-[13px] font-medium text-[#11111b] transition-colors hover:bg-[#94e2d5] disabled:bg-[#313244] disabled:text-[#45475a]"
        >
          Send
        </button>
      </form>
    </footer>
  );
}

export default ChatComposer;
