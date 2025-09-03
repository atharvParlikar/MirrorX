"use app";

import KlineChart from "@/components/Chart";
import { CandlestickData } from "lightweight-charts";

export default function Home() {
  const sampleData = [
    { time: 1672531200, open: 100, high: 110, low: 95, close: 105 },
    { time: 1672617600, open: 105, high: 120, low: 102, close: 115 },
    { time: 1672704000, open: 115, high: 118, low: 100, close: 102 },
  ] as CandlestickData[];

  return (
    <div>
      <KlineChart data={sampleData} />
    </div>
  );
}
