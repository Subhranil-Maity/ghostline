export type MessageSender = "Me" | "Remote";

export type ChatEntry = {
  uuid: string;
  content: string;
  timestamp: number;
  sender: MessageSender;
};

export type MessageEventPayload = {
  peer_id: string;
  message: ChatEntry;
};

export type ConnectionEventPayload = {
  connection_id: string;
};
