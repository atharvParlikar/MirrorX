let lastKnownBuyToSellRatio = 1.0; // more than 1 := more buys ; less than 1 := more sells
const SPREAD_CONSTANT = 0.001;

export function getBuyPrice(price: number) {
  return price + (lastKnownBuyToSellRatio * SPREAD_CONSTANT * price);
}

export function getSellPrice(price: number) {
  return price - (lastKnownBuyToSellRatio * SPREAD_CONSTANT * price);
}
