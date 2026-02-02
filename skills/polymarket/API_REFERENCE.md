# Polymarket API Reference

Complete API documentation for Polymarket integration with StarkBot.

## Base URLs

| Environment | Gamma API | CLOB API | Data API |
|-------------|-----------|----------|----------|
| **Production** | `https://gamma-api.polymarket.com` | `https://clob.polymarket.com` | `https://data-api.polymarket.com` |
| **Development** | `https://gamma-api-test.polymarket.com` | `https://clob-test.polymarket.com` | `https://data-api-test.polymarket.com` |

## Authentication

### API Key (Required for Trading)
```bash
# Add to headers
Authorization: Bearer YOUR_POLYMARKET_API_KEY

# Get API key from Polymarket Builders Program
# Apply at: https://polymarket.com/developers
```

### SIWE (Sign In With Ethereum)
For user account access:
```bash
# 1. Get nonce
curl "https://clob.polymarket.com/nonce"

# 2. Sign message with wallet
# Message: "Sign in to Polymarket: {nonce}"

# 3. Verify signature
curl -X POST "https://clob.polymarket.com/verify" \
  -d '{"message": "Sign in to Polymarket: {nonce}", "signature": "0x..."}'
```

## Market Data Endpoints

### Get Markets
```http
GET /events
```

**Parameters:**
- `active` (boolean): Filter active markets
- `closed` (boolean): Include closed markets  
- `limit` (number): Max results (default: 20, max: 100)
- `offset` (number): Pagination offset
- `order` (string): Sort field (volume, created, startTime)
- `ascending` (boolean): Sort direction
- `tag_id` (number): Filter by category tag
- `series_id` (number): Filter by sports series
- `slug` (string): Filter by market slug

**Response:**
```json
{
  "events": [
    {
      "id": "0x1234567890abcdef",
      "title": "Will Bitcoin reach $100k by 2025?",
      "slug": "will-bitcoin-reach-100k-by-2025",
      "description": "Bitcoin price reaches $100,000 USD before January 1, 2025",
      "startDate": "2024-01-01T00:00:00Z",
      "endDate": "2025-01-01T00:00:00Z",
      "image": "https://...",
      "icon": "https://...",
      "tags": [21, 100],
      "series_id": null,
      "active": true,
      "closed": false,
      "closed_date": null,
      "volume": "1250000.50",
      "liquidity": "500000.25",
      "outcomes": ["Yes", "No"],
      "outcomePrices": ["0.65", "0.35"],
      "created_at": "2024-01-01T00:00:00Z",
      "updated_at": "2024-01-15T10:30:00Z"
    }
  ],
  "next_offset": 20,
  "total": 150
}
```

### Get Market by ID
```http
GET /markets/{market_id}
```

**Response:**
```json
{
  "market": {
    "id": "0x1234567890abcdef",
    "condition_id": "0xabcdef1234567890",
    "slug": "will-bitcoin-reach-100k-by-2025",
    "title": "Will Bitcoin reach $100k by 2025?",
    "description": "...",
    "outcomes": ["Yes", "No"],
    "tokens": [
      {
        "id": "0x111...",
        "outcome": "Yes",
        "price": "0.65"
      },
      {
        "id": "0x222...", 
        "outcome": "No",
        "price": "0.35"
      }
    ],
    "volume": "1250000.50",
    "spread": "0.02",
    "liquidity": "500000.25",
    "status": "active",
    "resolution_date": "2025-01-01T00:00:00Z",
    "resolution_source": "CoinGecko BTC/USD"
  }
}
```

### Get Tags/Categories
```http
GET /tags
```

**Response:**
```json
{
  "tags": [
    {
      "id": 2,
      "name": "Politics",
      "slug": "politics",
      "description": "Political events and elections"
    },
    {
      "id": 21,
      "name": "Crypto",
      "slug": "crypto", 
      "description": "Cryptocurrency and blockchain"
    }
  ]
}
```

## Trading Endpoints

### Place Order
```http
POST /orders
```

**Request:**
```json
{
  "market": "0x1234567890abcdef",
  "side": "buy",
  "type": "limit",
  "size": "100",
  "price": "0.65",
  "time_in_force": "gtc",
  "post_only": false,
  "reduce_only": false
}
```

**Parameters:**
- `market` (string): Market ID
- `side` (string): `buy` or `sell`
- `type` (string): `market`, `limit`, `stop`
- `size` (string): Order size in shares
- `price` (string): Limit price (required for limit orders)
- `time_in_force` (string): `gtc` (good till cancel), `ioc` (immediate or cancel), `fok` (fill or kill)
- `post_only` (boolean): Post-only order (maker only)
- `reduce_only` (boolean): Reduce-only order

**Response:**
```json
{
  "order": {
    "id": "0xabc123...",
    "market": "0x1234567890abcdef",
    "side": "buy",
    "type": "limit", 
    "size": "100",
    "price": "0.65",
    "filled_size": "0",
    "status": "open",
    "created_at": "2024-01-15T10:30:00Z",
    "updated_at": "2024-01-15T10:30:00Z"
  }
}
```

### Get Orders
```http
GET /orders
```

**Parameters:**
- `status` (string): `open`, `filled`, `cancelled`
- `market` (string): Filter by market ID
- `limit` (number): Max results (default: 50, max: 100)
- `offset` (number): Pagination offset

### Cancel Order
```http
DELETE /orders/{order_id}
```

### Get Orderbook
```http
GET /book
```

**Parameters:**
- `token_id` (string): Token ID for the outcome
- `depth` (number): Orderbook depth (default: 10)

**Response:**
```json
{
  "bids": [
    {"price": "0.64", "size": "500"},
    {"price": "0.63", "size": "1000"}
  ],
  "asks": [
    {"price": "0.66", "size": "750"},
    {"price": "0.67", "size": "1200"}
  ],
  "spread": "0.02",
  "last_price": "0.65"
}
```

## Portfolio Endpoints

### Get Positions
```http
GET /positions
```

**Response:**
```json
{
  "positions": [
    {
      "id": "0xpos123...",
      "market_id": "0x1234567890abcdef",
      "market_title": "Will Bitcoin reach $100k by 2025?",
      "token_id": "0x111...",
      "outcome": "Yes",
      "size": "250",
      "avg_price": "0.60",
      "current_price": "0.65",
      "unrealized_pnl": "12.50",
      "realized_pnl": "0",
      "total_value": "162.50"
    }
  ],
  "total_value": "162.50",
  "total_unrealized_pnl": "12.50",
  "total_realized_pnl": "0"
}
```

### Get Trades
```http
GET /trades
```

**Parameters:**
- `market` (string): Filter by market ID
- `limit` (number): Max results (default: 50, max: 100)
- `offset` (number): Pagination offset
- `start_time` (string): Start time (ISO 8601)
- `end_time` (string): End time (ISO 8601)

## WebSocket API

### Connection
```javascript
const ws = new WebSocket('wss://ws-subscriptions-clob.polymarket.com');
```

### Subscribe to Market Updates
```javascript
ws.send(JSON.stringify({
  "type": "subscribe",
  "channel": "market",
  "market": "0x1234567890abcdef"
}));
```

### Subscribe to Order Updates
```javascript
ws.send(JSON.stringify({
  "type": "subscribe", 
  "channel": "orders",
  "api_key": "YOUR_POLYMARKET_API_KEY"
}));
```

### Message Format
```json
{
  "type": "update",
  "channel": "market",
  "data": {
    "market_id": "0x1234567890abcdef",
    "outcome_prices": ["0.65", "0.35"],
    "volume": "1250100.50",
    "timestamp": "2024-01-15T10:35:00Z"
  }
}
```

## Error Codes

| Code | Message | Description |
|------|---------|-------------|
| 400 | `invalid_request` | Invalid request parameters |
| 401 | `unauthorized` | Invalid or missing API key |
| 403 | `forbidden` | Access denied |
| 404 | `not_found` | Resource not found |
| 409 | `conflict` | Order conflict (e.g., post-only would cross) |
| 422 | `validation_error` | Validation failed |
| 429 | `rate_limit_exceeded` | Too many requests |
| 500 | `internal_error` | Server error |

## Rate Limits

- **Market Data**: 100 requests/minute per IP
- **Trading API**: 60 requests/minute per API key  
- **WebSocket**: 10 connections per IP
- **Portfolio**: 120 requests/minute per API key

## Best Practices

### Caching
```bash
# Cache market data responses
# Prices update every few seconds, no need to poll constantly
# Cache for 5-10 seconds depending on use case
```

### Batching
```bash
# Get multiple markets in one request
curl "https://gamma-api.polymarket.com/markets?ids=id1,id2,id3"
```

### Error Handling
```javascript
// Retry with exponential backoff
const retryWithBackoff = async (fn, retries = 3) => {
  for (let i = 0; i < retries; i++) {
    try {
      return await fn();
    } catch (error) {
      if (i === retries - 1) throw error;
      await new Promise(resolve => setTimeout(resolve, Math.pow(2, i) * 1000));
    }
  }
};
```

### WebSocket Reconnection
```javascript
// Auto-reconnect on disconnect
ws.on('close', () => {
  setTimeout(() => connectWebSocket(), 5000);
});
```

This reference covers the core Polymarket API endpoints needed for trading integration with StarkBot.