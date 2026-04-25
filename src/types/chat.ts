export type MessageSender = "Me" | "Remote";
export type PeerStatus = "Connected" | "Disconnected";

export type ChatMessage = {
  uuid: string;
  content: string;
  timestamp: number;
  sender: MessageSender;
};

export type HistoryEntry =
  | { SimpleTextMessage: ChatMessage }
  | { PeerStatusUpdated: PeerStatus };

export type MessageEventPayload = {
  peer_id: string;
  message: HistoryEntry;
};

export type ConnectionEventPayload = {
  connection_id: string;
};
