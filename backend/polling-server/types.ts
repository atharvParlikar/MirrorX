export type MarkPriceWsMessage = {
  stream: string,
  data: {
    e: string,
    E: number,
    s: string,
    p: string,
    P: string,
    i: string,
    r: string,
    T: number
  }
};

export type PriceUpdate = {
  symbol: string,
  buy: number,
  sell: number,
};
