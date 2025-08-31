package main

import (
	"bytes"
	"encoding/json"
	"fmt"
	"log"
	"math/rand"
	"net/http"
	"strings"
	"sync"
	"sync/atomic"
	"time"
)

type SignupRequest struct {
	Username string `json:"username"`
	Password string `json:"password"`
}

type GenericResponse struct {
	Message string `json:"message"`
}

type OpenOrderRequest struct {
	Qty        float64  `json:"qty"`
	Asset      string   `json:"asset"`
	Margin     *float64 `json:"margin,omitempty"`
	StopLoss   *float64 `json:"stop_loss,omitempty"`
	TakeProfit *float64 `json:"take_profit,omitempty"`
	Leverage   *float64 `json:"leverage,omitempty"`
}

type OpenOrderResponse struct {
	OrderID string `json:"order_id"`
}

type CloseOrderRequest struct {
	OrderID string `json:"order_id"`
}

type Bot struct {
	ID       int
	Username string
	UserID   string
	OrderID  string
	Client   *http.Client
}

type BenchmarkStats struct {
	TotalBots        int
	SuccessfulLogins int64
	TotalRequests    int64
	SuccessfulOpens  int64
	SuccessfulCloses int64
	FailedRequests   int64
	StartTime        time.Time
	EndTime          time.Time
}

const (
	BASE_URL = "http://localhost:8000" // Change this to your server URL
)

func main() {
	// Configuration
	numBots := 100
	testDuration := 8 * time.Minute // Total stress test duration

	fmt.Printf("Starting stress test with %d bots for %v duration\n", numBots, testDuration)

	stats := &BenchmarkStats{
		TotalBots: numBots,
		StartTime: time.Now(),
	}

	// Create bots
	bots := make([]*Bot, numBots)
	for i := 0; i < numBots; i++ {
		bots[i] = &Bot{
			ID:       i,
			Username: fmt.Sprintf("bot_%d", i),
			Client:   &http.Client{Timeout: 10 * time.Second},
		}
	}

	// Phase 1: Signup all bots (not measured for performance)
	fmt.Println("Setting up bots (signup phase)...")
	signupBots(bots, stats)

	// Phase 2: Continuous stress test - open and close positions rapidly
	fmt.Printf("Starting stress test for %v...\n", testDuration)
	fmt.Println("Live feed (O=open success, C=close success, X=failed request):")
	stats.StartTime = time.Now() // Reset timer for actual test

	stopChan := make(chan bool)

	// Start stress test goroutines
	var wg sync.WaitGroup
	for _, bot := range bots {
		if bot.UserID == "" {
			continue // Skip failed signups
		}

		wg.Add(1)
		go func(b *Bot) {
			defer wg.Done()
			stressTestBot(b, stats, stopChan)
		}(bot)
	}

	// Let it run for the specified duration
	time.Sleep(testDuration)
	close(stopChan)

	// Wait for all bots to finish their current operations
	wg.Wait()

	stats.EndTime = time.Now()
	printStats(stats)
}

func signupBots(bots []*Bot, stats *BenchmarkStats) {
	var wg sync.WaitGroup

	for _, bot := range bots {
		wg.Add(1)
		go func(b *Bot) {
			defer wg.Done()

			signupReq := SignupRequest{
				Username: b.Username,
				Password: "password123", // Simple password for all bots
			}

			jsonData, _ := json.Marshal(signupReq)
			resp, err := b.Client.Post(BASE_URL+"/signup", "application/json", bytes.NewBuffer(jsonData))

			if err != nil {
				log.Printf("Bot %d signup error: %v", b.ID, err)
				return
			}
			defer resp.Body.Close()

			if resp.StatusCode >= 400 {
				log.Printf("Bot %d signup failed with status: %d", b.ID, resp.StatusCode)
				return
			}

			var response GenericResponse
			json.NewDecoder(resp.Body).Decode(&response)

			// Treat the message as user_id/JWT
			b.UserID = response.Message
			atomic.AddInt64(&stats.SuccessfulLogins, 1)

			fmt.Printf("Bot %d signed up successfully, UserID: %s\n", b.ID, b.UserID)
		}(bot)
	}

	wg.Wait()
}

func stressTestBot(bot *Bot, stats *BenchmarkStats, stopChan chan bool) {
	// Randomize start time to spread out the bots
	initialDelay := time.Duration(rand.Intn(1000)) * time.Millisecond
	time.Sleep(initialDelay)

	for {
		select {
		case <-stopChan:
			return
		default:
			// Open position
			orderID := openPosition(bot, stats)
			if orderID != "" {
				// Random delay before closing (0-500ms for more realistic timing)
				closeDelay := time.Duration(rand.Intn(500)) * time.Millisecond
				time.Sleep(closeDelay)

				// Close position
				closePosition(bot, orderID, stats)
			}

			// Random delay between complete cycles (50-200ms)
			cycleDelay := time.Duration(50+rand.Intn(150)) * time.Millisecond
			time.Sleep(cycleDelay)
		}
	}
}

func openPosition(bot *Bot, stats *BenchmarkStats) string {
	// Small random BTC quantity
	qty := 0.001 + rand.Float64()*(0.01-0.001)

	orderReq := OpenOrderRequest{
		Qty:   qty,
		Asset: "BTC",
	}

	jsonData, _ := json.Marshal(orderReq)
	req, _ := http.NewRequest("POST", BASE_URL+"/order/open", bytes.NewBuffer(jsonData))
	req.Header.Set("Content-Type", "application/json")
	req.Header.Set("Authorization", "Bearer "+bot.UserID)

	atomic.AddInt64(&stats.TotalRequests, 1)
	resp, err := bot.Client.Do(req)

	if err != nil {
		atomic.AddInt64(&stats.FailedRequests, 1)
		fmt.Print("X")
		return ""
	}
	defer resp.Body.Close()

	if resp.StatusCode >= 400 {
		atomic.AddInt64(&stats.FailedRequests, 1)
		fmt.Print("X")
		return ""
	}

	var response OpenOrderResponse
	json.NewDecoder(resp.Body).Decode(&response)

	atomic.AddInt64(&stats.SuccessfulOpens, 1)
	fmt.Print("O")
	return response.OrderID
}

func closePosition(bot *Bot, orderID string, stats *BenchmarkStats) {
	closeReq := CloseOrderRequest{
		OrderID: orderID,
	}

	jsonData, _ := json.Marshal(closeReq)
	req, _ := http.NewRequest("POST", BASE_URL+"/order/close", bytes.NewBuffer(jsonData))
	req.Header.Set("Content-Type", "application/json")
	req.Header.Set("Authorization", "Bearer "+bot.UserID)

	atomic.AddInt64(&stats.TotalRequests, 1)
	resp, err := bot.Client.Do(req)

	if err != nil {
		atomic.AddInt64(&stats.FailedRequests, 1)
		fmt.Print("X")
		return
	}
	defer resp.Body.Close()

	if resp.StatusCode >= 400 {
		atomic.AddInt64(&stats.FailedRequests, 1)
		fmt.Print("X")
		return
	}

	atomic.AddInt64(&stats.SuccessfulCloses, 1)
	fmt.Print("C")
}

func printStats(stats *BenchmarkStats) {
	duration := stats.EndTime.Sub(stats.StartTime)

	fmt.Println("\n" + strings.Repeat("=", 60))
	fmt.Println("STRESS TEST RESULTS")
	fmt.Println(strings.Repeat("=", 60))
	fmt.Printf("Test Duration: %v\n", duration)
	fmt.Printf("Active Bots: %d/%d\n", stats.SuccessfulLogins, stats.TotalBots)
	fmt.Println()

	fmt.Println("PERFORMANCE METRICS:")
	fmt.Printf("  Total Requests: %d\n", stats.TotalRequests)
	fmt.Printf("  Successful Opens: %d\n", stats.SuccessfulOpens)
	fmt.Printf("  Successful Closes: %d\n", stats.SuccessfulCloses)
	fmt.Printf("  Failed Requests: %d\n", stats.FailedRequests)
	fmt.Printf("  Success Rate: %.1f%%\n",
		float64(stats.SuccessfulOpens+stats.SuccessfulCloses)/float64(stats.TotalRequests)*100)
	fmt.Println()

	fmt.Println("THROUGHPUT:")
	fmt.Printf("  Requests/sec: %.2f\n", float64(stats.TotalRequests)/duration.Seconds())
	fmt.Printf("  Opens/sec: %.2f\n", float64(stats.SuccessfulOpens)/duration.Seconds())
	fmt.Printf("  Closes/sec: %.2f\n", float64(stats.SuccessfulCloses)/duration.Seconds())
	fmt.Printf("  Successful req/sec: %.2f\n",
		float64(stats.SuccessfulOpens+stats.SuccessfulCloses)/duration.Seconds())

	fmt.Println(strings.Repeat("=", 60))
}
