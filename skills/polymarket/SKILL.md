---
name: polymarket
version: 1.0.0
description: "Trade on Polymarket prediction markets. Browse markets, get prices, place orders, and manage positions."
homepage: https://polymarket.com
metadata: {"emoji":"📊","category":"defi","api_base":"https://clob.polymarket.com","chain":"polygon"}
---

# Polymarket Trading

Trade on the world's largest prediction market. Bet on real-world events, from politics and crypto to sports and climate. All markets settle in USDC on Polygon.

**Base URLs:**
- **Gamma API** (market data): `https://gamma-api.polymarket.com`
- **CLOB API** (trading): `https://clob.polymarket.com`
- **Data API** (positions): `https://data-api.polymarket.com`

## Quick Start

### 1. Browse Markets
```bash
# Get active markets
curl "https://gamma-api.polymarket.com/events?active=true&closed=false&limit=10"

# Search by category
curl "https://gamma-api.polymarket.com/events?tag_id=21&active=true&closed=false"  # Crypto
curl "https://gamma-api.polymarket.com/events?tag_id=9&active=true&closed=false"   # Politics
curl "https://gamma-api.polymarket.com/events?tag_id=11&active=true&closed=false"  # Sports
```

### 2. Get Market Details
```bash
# Get specific market by slug
curl "https://gamma-api.polymarket.com/markets?slug=will-bitcoin-reach-100k-by-2025"

# Get by market ID
curl "https://gamma-api.polymarket.com/markets/0x1234567890abcdef"
```

### 3. Check Prices
```bash
# Get current price for outcome token
curl "https://clob.polymarket.com/price?token_id=TOKEN_ID&side=buy"

# Get orderbook depth
curl "https://clob.polymarket.com/book?token_id=TOKEN_ID"
```

## Market Discovery

### Categories & Tags
```bash
# Get all available tags
curl "https://gamma-api.polymarket.com/tags?limit=100"

# Popular categories:
# 2: Politics, 9: US Politics, 21: Crypto, 11: Sports, 100639: Sports Games
# 100640: NFL, 100641: NBA, 100642: MLB, 100643: Soccer
```

### Sports Markets
```bash
# Get sports leagues
curl "https://gamma-api.polymarket.com/sports"

# Get specific league events (NBA example)
curl "https://gamma-api.polymarket.com/events?series_id=10345&active=true&closed=false"

# Get today's games
curl "https://gamma-api.polymarket.com/events?series_id=10345&tag_id=100639&active=true&closed=false&order=startTime&ascending=true"
```

## Trading

### Authentication Required
Trading requires Polymarket account authentication. You'll need:
1. Polymarket account with linked wallet
2. API credentials (get from Polymarket Builders Program)
3. USDC balance on Polygon

### Place Orders
```bash
# Place a market order (requires auth)
curl -X POST "https://clob.polymarket.com/orders" \
  -H "Authorization: Bearer YOUR_POLYMARKET_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "market": "MARKET_ID",
    "side": "buy",
    "type": "market",
    "size": "100",
    "price": "0.65"
  }'

# Place limit order
curl -X POST "https://clob.polymarket.com/orders" \
  -H "Authorization: Bearer YOUR_POLYMARKET_API_KEY" \
  -H "Content-Type: application/json" \
  -d '{
    "market": "MARKET_ID",
    "side": "buy",
    "type": "limit",
    "size": "100",
    "price": "0.60"
  }'
```

### Manage Orders
```bash
# Get open orders
curl "https://clob.polymarket.com/orders" \
  -H "Authorization: Bearer YOUR_POLYMARKET_API_KEY"

# Cancel order
curl -X DELETE "https://clob.polymarket.com/orders/ORDER_ID" \
  -H "Authorization: Bearer YOUR_POLYMARKET_API_KEY"
```

## Positions & Portfolio

### Get Positions
```bash
# Get user positions (requires auth)
curl "https://data-api.polymarket.com/positions" \
  -H "Authorization: Bearer YOUR_POLYMARKET_API_KEY"

# Get position details
curl "https://data-api.polymarket.com/positions/POSITION_ID" \
  -H "Authorization: Bearer YOUR_POLYMARKET_API_KEY"
```

### Trade History
```bash
# Get trade history
curl "https://data-api.polymarket.com/trades" \
  -H "Authorization: Bearer YOUR_POLYMARKET_API_KEY" \
  -G \
  -d "limit=50" \
  -d "offset=0"
```

## Market Data Examples

### Crypto Market
```json
{
  "id": "123456",
  "title": "Will Bitcoin reach $100k by 2025?",
  "outcomes": ["Yes", "No"],
  "outcomePrices": ["0.65", "0.35"],
  "volume": "1250000",
  "liquidity": "500000"
}
```

### Political Market
```json
{
  "id": "789012",
  "title": "Who will win the 2024 US Presidential Election?",
  "outcomes": ["Democratic Candidate", "Republican Candidate", "Other"],
  "outcomePrices": ["0.52", "0.46", "0.02"],
  "volume": "25000000",
  "liquidity": "8000000"
}
```

### Sports Market
```json
{
  "id": "345678",
  "title": "Lakers vs Warriors - Who will win?",
  "outcomes": ["Lakers", "Warriors"],
  "outcomePrices": ["0.45", "0.55"],
  "startTime": "2024-01-15T20:00:00Z",
  "series_id": "10345"
}
```

## Advanced Features

### WebSocket Real-time Data
```bash
# Connect to WebSocket for real-time updates
wscat -c wss://ws-subscriptions-clob.polymarket.com

# Subscribe to market updates
{"type": "subscribe", "channel": "market", "market": "MARKET_ID"}
```

### Builders Program
For automated trading applications, join the Polymarket Builders Program:
- Apply at polymarket.com/developers
- Get API keys for programmatic trading
- Access to relayer for gasless transactions
- Order attribution for fee rebates

## Error Handling

Common API responses:
- `200`: Success
- `400`: Bad request (invalid parameters)
- `401`: Unauthorized (invalid API key)
- `403`: Forbidden (restricted access)
- `404`: Market not found
- `429`: Rate limit exceeded
- `500`: Server error

## Rate Limits

- Market data: 100 requests per minute
- Trading API: 60 requests per minute
- WebSocket: 10 connections per IP

## Best Practices

1. **Cache market data** - prices update every few seconds
2. **Check market status** before trading (active/closed/settled)
3. **Use limit orders** for better price control
4. **Monitor gas prices** on Polygon for cost optimization
5. **Start small** - test with small amounts first
6. **Diversify** - don't put all funds in one market

## Risk Warning

Prediction markets are speculative. Prices reflect probabilities, not guarantees. Only trade with funds you can afford to lose. Markets can be volatile and illiquid. Always do your own research.

## Support

- Documentation: docs.polymarket.com
- Discord: discord.gg/polymarket
- Twitter: @polymarket
- API Status: status.polymarket.com