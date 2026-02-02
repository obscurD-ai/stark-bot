# Polymarket Trading Heartbeat 📊

*Monitor markets, track positions, and identify trading opportunities. Run every 15-30 minutes for active trading.*

---

## 1. Check Market Updates

### Get Active Markets
```bash
# Fetch trending markets
curl "https://gamma-api.polymarket.com/events?active=true&closed=false&limit=20&order=volume&descending=true"

# Check your watched markets (store these in memory)
curl "https://gamma-api.polymarket.com/markets?ids=WATCHED_MARKET_IDS"
```

### Monitor Price Changes
```bash
# Get significant movers (>5% price change in 24h)
curl "https://gamma-api.polymarket.com/events?active=true&closed=false&price_change=5"
```

---

## 2. Update Portfolio Tracking

### Check Positions
```bash
# Get current positions (if authenticated)
curl "https://data-api.polymarket.com/positions" \
  -H "Authorization: Bearer YOUR_POLYMARKET_API_KEY"

# Calculate P&L
curl "https://data-api.polymarket.com/pnl" \
  -H "Authorization: Bearer YOUR_POLYMARKET_API_KEY"
```

### Track Open Orders
```bash
# Monitor active orders
curl "https://clob.polymarket.com/orders?status=open" \
  -H "Authorization: Bearer YOUR_POLYMARKET_API_KEY"

# Check for filled orders
curl "https://clob.polymarket.com/orders?status=filled&limit=10" \
  -H "Authorization: Bearer YOUR_POLYMARKET_API_KEY"
```

---

## 3. Identify Opportunities

### Arbitrage Detection
```bash
# Compare prices across related markets
curl "https://gamma-api.polymarket.com/events?tag_id=9&active=true" | \
  jq '.[] | select(.title | contains("Trump") or contains("Biden")) | {title, outcomes, outcomePrices}'
```

### Volume Spikes
```bash
# Markets with unusual volume
curl "https://gamma-api.polymarket.com/events?active=true&volume_spike=3"
```

### New Listings
```bash
# Recently created markets
curl "https://gamma-api.polymarket.com/events?active=true&created_after=1day"
```

---

## 4. Risk Management

### Position Limits
```bash
# Check total exposure by category
curl "https://data-api.polymarket.com/exposure" \
  -H "Authorization: Bearer YOUR_POLYMARKET_API_KEY"

# Set alerts for concentration risk
# If any single position >20% of portfolio → consider reducing
```

### Stop Loss Monitoring
```bash
# Check markets near your stop levels
# If price moved against you by >X% → consider closing
```

---

## 5. Market Analysis

### News & Events
```bash
# Check for upcoming market-moving events
curl "https://gamma-api.polymarket.com/events?closing_within=24h"

# Monitor settled markets for resolution patterns
curl "https://gamma-api.polymarket.com/events?settled=true&limit=10"
```

### Correlation Analysis
```bash
# Track related markets moving together
# Political markets often correlate
# Crypto markets move with BTC/ETH
```

---

## 6. Trading Actions

### If You Find Opportunities:

#### Place Orders
```bash
# Buy undervalued outcome
curl -X POST "https://clob.polymarket.com/orders" \
  -H "Authorization: Bearer YOUR_POLYMARKET_API_KEY" \
  -d '{"market": "MARKET_ID", "side": "buy", "type": "limit", "size": "50", "price": "0.45"}'

# Sell overvalued outcome
curl -X POST "https://clob.polymarket.com/orders" \
  -H "Authorization: Bearer YOUR_POLYMARKET_API_KEY" \
  -d '{"market": "MARKET_ID", "side": "sell", "type": "limit", "size": "50", "price": "0.85"}'
```

#### Adjust Existing Orders
```bash
# Cancel and replace with better prices
curl -X DELETE "https://clob.polymarket.com/orders/ORDER_ID" \
  -H "Authorization: Bearer YOUR_POLYMARKET_API_KEY"
```

---

## 7. Record Keeping

### Trade Log
```json
{
  "timestamp": "2024-01-15T10:30:00Z",
  "market_id": "0x123...",
  "action": "bought",
  "outcome": "Yes",
  "size": 100,
  "price": 0.65,
  "reason": "Undervalued based on polling data"
}
```

### Performance Tracking
```bash
# Weekly P&L summary
curl "https://data-api.polymarket.com/performance?period=7d" \
  -H "Authorization: Bearer YOUR_POLYMARKET_API_KEY"
```

---

## 8. Schedule & State

**Run every 15-30 minutes during market hours.** Track state:
```json
{
  "lastHeartbeat": "2024-01-15T10:30:00Z",
  "watchedMarkets": ["market_id_1", "market_id_2"],
  "positions": {},
  "orders": {},
  "alerts": [],
  "tradingEnabled": false
}
```

**Priority order each heartbeat:**
1. Check portfolio health (positions, P&L)
2. Monitor watched markets for significant moves
3. Scan for new opportunities (arbitrage, volume spikes)
4. Review and adjust open orders
5. Update trade log and performance metrics
6. Set alerts for next period

---

## Trading Strategy Ideas

### Momentum Trading
- Identify markets with strong directional movement
- Ride the trend but watch for reversals
- Use volume confirmation

### Mean Reversion
- Look for overextended moves (>2 standard deviations)
- Bet on normalization
- Requires tight risk management

### Event-Driven
- Trade around major events (elections, earnings, etc.)
- Monitor news and social sentiment
- Exit before event resolution

### Arbitrage
- Find mispriced related markets
- Requires quick execution
- Small but consistent profits

---

## Risk Management Rules

1. **Never risk more than 5% on single market**
2. **Set stop losses at 20% loss**
3. **Diversify across categories**
4. **Avoid illiquid markets** (<$10k daily volume)
5. **Don't chase losses**
6. **Take profits systematically**

---

## Remember

- **Markets are efficient** - edge is small and temporary
- **Information advantage** is key - stay informed
- **Risk management** beats prediction skills
- **Consistency** > home runs
- **Track everything** - data improves decisions

*Trade smart, manage risk, let probabilities work in your favor. 📊*