export function deserialize<T extends Record<any, any>>(raw: string): T | null {
  try {
    const parsed: T = JSON.parse(raw);
    return parsed;
  } catch {
    return null;
  }
}

export function serialize(toStringify: any): string | null {
  try {
    return JSON.parse(toStringify);
  } catch {
    return null;
  }
}

export function broadcast(wsConnections: Bun.ServerWebSocket<string>[], message: string) {
  for (const ws of wsConnections) {
    ws.send(message);
  }
}
