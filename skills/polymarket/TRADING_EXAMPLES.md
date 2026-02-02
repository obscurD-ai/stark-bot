# Polymarket Trading Examples

Real examples of trading strategies and API calls for common scenarios.

## Example 1: Simple Directional Bet

**Scenario**: You think Bitcoin will reach $100k by 2025, current "Yes" price is 0.35 (35% chance)

### Step 1: Research the Market
```bash
# Get market details
curl "https://gamma-api.polymarket.com/markets?slug=will-bitcoin-reach-100k-by-2025"

# Check orderbook depth
curl "https://clob.polymarket.com/book?token_id=YES_TOKEN_ID"
```

### Step 2: Place Buy Order
```bash
# Buy $100 worth of "Yes" at 0.40 limit (better than market price)
curl -X POST "https://clob.polymarket.com/orders" \
  -H "Authorization: Bearer YOUR_POLYMARKET_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "market": "0x1234567890abcdef",
    "side": "buy",
    "type": "limit",
    "size": "250",  // $100 / 0.40 = 250 shares
    "price": "0.40"
  }'
```

### Step 3: Monitor Position
```bash
# Check if order filled
curl "https://clob.polymarket.com/orders/ORDER_ID" \
  -H "Authorization: Bearer YOUR_POLYMARKET_API_KEY"

# Track P&L
curl "https://data-api.polymarket.com/positions" \
  -H "Authorization: Bearer YOUR_POLYMARKET_API_KEY"
```

---

## Example 2: Arbitrage Opportunity

**Scenario**: Related political markets showing price inconsistency

### Step 1: Find Mispricing
```bash
# Get all Trump-related markets
curl "https://gamma-api.polymarket.com/events?active=true&closed=false" | \
  jq '.[] | select(.title | contains("Trump") and contains("2024")) | {title, outcomes, outcomePrices}'

# Example findings:
# Market A: "Trump wins 2024" - Yes at 0.48
# Market B: "Republican wins 2024" - Yes at 0.44
# These should be closer in price
```

### Step 2: Execute Arbitrage
```bash
# Buy undervalued (Republican wins)
curl -X POST "https://clob.polymarket.com/orders" \
  -H "Authorization: Bearer YOUR_POLYMARKET_API_KEY" \
  -d '{"market": "REP_MARKET_ID", "side": "buy", "type": "limit", "size": "500", "price": "0.44"}'

# Sell overvalued (Trump wins) if you can short, or buy "No" on Trump
curl -X POST "https://clob.polymarket.com/orders" \
  -H "Authorization: Bearer YOUR_POLYMARKET_API_KEY" \
  -d '{"market": "TRUMP_MARKET_ID", "side": "buy", "type": "limit", "size": "500", "price": "0.52"}'
```

---

## Example 3: Event-Driven Trading

**Scenario**: Major debate tonight, expect volatility in election markets

### Step 1: Pre-Event Setup
```bash
# Check current positioning
curl "https://data-api.polymarket.com/positions" \
  -H "Authorization: Bearer YOUR_POLYMARKET_API_KEY"

# Get debate market details
curl "https://gamma-api.polymarket.com/markets?slug=who-will-win-tonights-debate"
```

### Step 2: Place Conditional Orders
```bash
# Buy if price drops to attractive level
curl -X POST "https://clob.polymarket.com/orders" \
  -H "Authorization: Bearer YOUR_POLYMARKET_API_KEY" \
  -d '{
    "market": "DEBATE_MARKET_ID",
    "side": "buy",
    "type": "limit",
    "size": "200",
    "price": "0.35",  // Buy if drops to 35%
    "post_only": true
  }'
```

### Step 3: Post-Event Management
```bash
# Monitor for significant moves
curl "https://clob.polymarket.com/price?token_id=DEBATE_TOKEN&side=buy"

# Take profits if moved in your favor
if [ "$current_price" -gt "0.60" ]; then
  curl -X POST "https://clob.polymarket.com/orders" \
    -H "Authorization: Bearer YOUR_POLYMARKET_API_KEY" \
    -d '{"market": "DEBATE_MARKET_ID", "side": "sell", "type": "market", "size": "200"}'
fi
```

---

## Example 4: Dollar-Cost Averaging

**Scenario**: Building position over time in long-term market

### Automated Script
```bash
#!/bin/bash
# DCA into "Will AI achieve AGI by 2030" market

MARKET_ID="0xabcdef1234567890"
BUY_AMOUNT=50  # $50 per purchase
FREQUENCY="weekly"

# Check current price
current_price=$(curl -s "https://clob.polymarket.com/price?token_id=$MARKET_ID&side=buy" | jq -r '.price')

# Calculate shares to buy
shares=$(echo "$BUY_AMOUNT / $current_price" | bc -l)

# Place order
curl -X POST "https://clob.polymarket.com/orders" \
  -H "Authorization: Bearer YOUR_POLYMARKET_API_KEY" \
  -d "{
    \"market\": \"$MARKET_ID\",
    \"side\": \"buy\",
    \"type\": \"limit\",
    \"size\": \"$shares\",
    \"price\": \"$current_price\",
    \"post_only\": true
  }"

echo "Bought $shares shares at $current_price price"
```

---

## Example 5: Risk Management

**Scenario**: Portfolio getting too concentrated in single market

### Step 1: Check Concentration
```bash
# Get portfolio breakdown
curl "https://data-api.polymarket.com/positions" \
  -H "Authorization: Bearer YOUR_POLYMARKET_API_KEY" | \
  jq '.positions | group_by(.market_id) | map({market_id: .[0].market_id, total_value: map(.value) | add})'

# If any position > 30% of total, reduce it
```

### Step 2: Reduce Position
```bash
# Sell portion of large position
large_position_id="LARGE_POSITION_ID"
sell_amount=100  // Sell 100 shares

curl -X POST "https://clob.polymarket.com/orders" \
  -H "Authorization: Bearer YOUR_POLYMARKET_API_KEY" \
  -d '{
    "market": "LARGE_MARKET_ID",
    "side": "sell",
    "type": "limit",
    "size": "100",
    "price": "market"  // Accept current market price
  }'
```

---

## Example 6: News-Based Trading

**Scenario**: Breaking news affects market, need to act quickly

### Step 1: Get News Impact
```bash
# Check which markets might be affected by news
# Example: Climate news affects climate markets
curl "https://gamma-api.polymarket.com/events?tag_id=25&active=true" | \
  jq '.[] | select(.title | contains("climate")) | {title, outcomePrices}'
```

### Step 2: Quick Execution
```bash
# Place market order for immediate execution
# (Accept worse price for speed)
curl -X POST "https://clob.polymarket.com/orders" \
  -H "Authorization: Bearer YOUR_POLYMARKET_API_KEY" \
  -d '{
    "market": "CLIMATE_MARKET_ID",
    "side": "buy",
    "type": "market",
    "size": "150",
    "time_in_force": "ioc"  // Immediate or cancel
  }'
```

---

## Common Error Handling

### Insufficient Balance
```json
{
  "error": "insufficient_balance",
  "message": "Not enough USDC for this order"
}
# Solution: Check USDC balance, deposit more if needed
```

### Market Closed
```json
{
  "error": "market_closed",
  "message": "This market is no longer active"
}
# Solution: Check market status before trading
```

### Price Slippage
```json
{
  "error": "price_slippage",
  "message": "Price moved too much, order rejected"
}
# Solution: Use limit orders instead of market orders
```

### Rate Limiting
```json
{
  "error": "rate_limit",
  "message": "Too many requests, try again later"
}
# Solution: Add delays between requests, use WebSocket for real-time data
```

---

## Best Practices Summary

1. **Always use limit orders** when possible for better price control
2. **Check orderbook depth** before placing large orders
3. **Start with small sizes** to test strategies
4. **Monitor your positions** regularly
5. **Set stop losses** at predetermined levels
6. **Keep detailed records** of all trades
7. **Diversify across markets** to manage risk
8. **Stay informed** about market-moving events
9. **Test strategies** with small amounts first
10. **Have a plan** before entering any trade

*Start simple, learn from experience, and scale up gradually. Happy trading! 📊*