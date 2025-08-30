export type LiquidationMessage = {
  positions: {
    user_id: string,
    position_id: string
  }[]
}

export type PriceUpdates = {
  bid: number,
  ask: number
};
