import { RefObject } from "react";

import { ChatEntry } from "../types/chat";

type MessageListProps = {
  messages: ChatEntry[];
  scrollRef: RefObject<HTMLDivElement | null>;
  onScroll: () => void;
};

function formatTimestamp(timestamp: number) {
  return new Date(timestamp).toLocaleTimeString([], {
    hour: "numeric",
    minute: "2-digit",
    hour12: true,
  });
}

function MessageList({ messages, scrollRef, onScroll }: MessageListProps) {
  return (
    <div
      ref={scrollRef}
      onScroll={onScroll}
      className="themed-scrollbar min-h-0 flex-1 overflow-y-auto px-5 py-5"
    >
      <div className="mx-auto flex w-full max-w-3xl flex-col gap-5">
        {messages.length === 0 ? (
          <p className="text-xs leading-relaxed text-[#45475a]">
            No messages yet. Select a connection, refresh, or send a message.
          </p>
        ) : (
          messages.map((entry) => {
            const isMe = entry.sender === "Me";
            return (
              <article key={entry.uuid} className="flex gap-3">
                <span
                  className={`mt-0.5 w-0.5 shrink-0 self-stretch rounded-sm ${
                    isMe ? "bg-[#89b4fa]" : "bg-[#fab387]"
                  }`}
                />
                <div className="min-w-0">
                  <p
                    className={`mb-1 text-[10px] uppercase tracking-[0.16em] ${
                      isMe ? "text-[#89b4fa]" : "text-[#fab387]"
                    }`}
                  >
                    {isMe ? "You" : "Remote"}
                    <span className="ml-2 font-normal tracking-[0.08em] text-[#7f849c]">
                      {formatTimestamp(entry.timestamp)}
                    </span>
                  </p>
                  <p className="whitespace-pre-wrap break-words text-[13px] leading-7 text-[#bac2de]">
                    {entry.content}
                  </p>
                </div>
              </article>
            );
          })
        )}
      </div>
    </div>
  );
}

export default MessageList;
