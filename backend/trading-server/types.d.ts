type OpenOrderRequest = {
  asset: string;
  margin: number;
  stop_loss?: number;
  take_profit?: number;
  leverage?: number;
};

