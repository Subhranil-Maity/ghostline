export type ChatEntry = {
  from: string;
  message: string;
};

export type MessageEventPayload = {
  connection_id: string;
  from: string;
  message: string;
};

export type ConnectionEventPayload = {
  connection_id: string;
};
