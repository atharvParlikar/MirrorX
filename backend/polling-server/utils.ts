const SPREAD_CONSTANT = 0.001; // 1% on each side := 2% total

export function getBuyPrice(price: number) {
  return price + (SPREAD_CONSTANT * price);
}

export function getSellPrice(price: number) {
  return price - (SPREAD_CONSTANT * price);
}
