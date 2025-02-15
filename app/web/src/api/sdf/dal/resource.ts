export enum ResourceHealth {
  Ok = "ok",
  Warning = "warning",
  Error = "error",
  Unknown = "unknown",
}

export enum ResourceStatus {
  Pending = "pending",
  InProgress = "inProgress",
  Created = "created",
  Failed = "failed",
  Deleted = "deleted",
}

export interface Resource {
  data: unknown;
  status: ResourceHealth;
  message: string | null;
  logs: string[];
  lastSynced?: string;
}

export interface OutputStream {
  line: string;
  stream: string;
  level: string;
}
