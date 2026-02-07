---
name: railway
description: "Manage Railway infrastructure - deploy services, manage environment variables, and monitor deployments."
version: 1.0.0
author: starkbot
homepage: https://railway.com
metadata: {"requires_auth": true, "clawdbot":{"emoji":"🚂"}}
requires_tools: [web_fetch, api_keys_check]
tags: [development, devops, railway, infrastructure, deployment, hosting]
---

# Railway Integration

Manage your Railway infrastructure via the GraphQL API. Deploy services, manage environment variables, check deployment status, and more.

## Authentication

**First, check if RAILWAY_API_TOKEN is configured:**
```tool:api_keys_check
key_name: RAILWAY_API_TOKEN
```

If not configured, ask the user to create an API token at https://railway.com/account/tokens and add it in Settings > API Keys.

---

## How to Use This Skill

All Railway API calls use the `web_fetch` tool to POST GraphQL queries to the Railway API:

- **URL**: `https://backboard.railway.com/graphql/v2`
- **Method**: POST
- **Headers**: `{"Authorization": "Bearer $RAILWAY_API_TOKEN", "Content-Type": "application/json"}`
- **extract_mode**: `"raw"` (Railway returns JSON, not HTML)

The `$RAILWAY_API_TOKEN` placeholder is automatically expanded from the stored API key.

---

## Operations

### 1. Verify Authentication

Check that the token is valid and see who you're authenticated as:

```tool:web_fetch
url: https://backboard.railway.com/graphql/v2
method: POST
headers: {"Authorization": "Bearer $RAILWAY_API_TOKEN", "Content-Type": "application/json"}
body: {"query": "{ me { name email } }"}
extract_mode: raw
```

### 2. List Projects

Get all projects with their IDs, names, and environments:

```tool:web_fetch
url: https://backboard.railway.com/graphql/v2
method: POST
headers: {"Authorization": "Bearer $RAILWAY_API_TOKEN", "Content-Type": "application/json"}
body: {"query": "{ projects { edges { node { id name description environments { edges { node { id name } } } } } } }"}
extract_mode: raw
```

### 3. Get Project Details

Get services and environments for a specific project (replace `PROJECT_ID`):

```tool:web_fetch
url: https://backboard.railway.com/graphql/v2
method: POST
headers: {"Authorization": "Bearer $RAILWAY_API_TOKEN", "Content-Type": "application/json"}
body: {"query": "query { project(id: \"PROJECT_ID\") { id name services { edges { node { id name } } } environments { edges { node { id name } } } } }"}
extract_mode: raw
```

### 4. Get Deployments

Get recent deployments with status for a service (replace `SERVICE_ID` and `ENVIRONMENT_ID`):

```tool:web_fetch
url: https://backboard.railway.com/graphql/v2
method: POST
headers: {"Authorization": "Bearer $RAILWAY_API_TOKEN", "Content-Type": "application/json"}
body: {"query": "query { deployments(first: 10, input: { serviceId: \"SERVICE_ID\", environmentId: \"ENVIRONMENT_ID\" }) { edges { node { id status createdAt staticUrl } } } }"}
extract_mode: raw
```

### 5. Trigger Redeploy

**IMPORTANT: Confirm with the user before triggering a redeploy.**

Redeploy the latest deployment for a service (replace `SERVICE_ID` and `ENVIRONMENT_ID`):

```tool:web_fetch
url: https://backboard.railway.com/graphql/v2
method: POST
headers: {"Authorization": "Bearer $RAILWAY_API_TOKEN", "Content-Type": "application/json"}
body: {"query": "mutation { serviceInstanceRedeploy(serviceId: \"SERVICE_ID\", environmentId: \"ENVIRONMENT_ID\") }"}
extract_mode: raw
```

### 6. Get Environment Variables

Get variables for a service in an environment (replace `SERVICE_ID`, `ENVIRONMENT_ID`, and `PROJECT_ID`):

```tool:web_fetch
url: https://backboard.railway.com/graphql/v2
method: POST
headers: {"Authorization": "Bearer $RAILWAY_API_TOKEN", "Content-Type": "application/json"}
body: {"query": "query { variables(serviceId: \"SERVICE_ID\", environmentId: \"ENVIRONMENT_ID\", projectId: \"PROJECT_ID\") }"}
extract_mode: raw
```

The response is a JSON object where keys are variable names and values are their values.

### 7. Set Environment Variables

**IMPORTANT: Confirm with the user before modifying environment variables.**

Upsert variables for a service (replace IDs and the variables object):

```tool:web_fetch
url: https://backboard.railway.com/graphql/v2
method: POST
headers: {"Authorization": "Bearer $RAILWAY_API_TOKEN", "Content-Type": "application/json"}
body: {"query": "mutation { variableCollectionUpsert(input: { serviceId: \"SERVICE_ID\", environmentId: \"ENVIRONMENT_ID\", projectId: \"PROJECT_ID\", variables: { KEY_NAME: \"VALUE\" } }) }"}
extract_mode: raw
```

### 8. Get Service Domains

Get domains for a service (replace `SERVICE_ID`, `ENVIRONMENT_ID`, and `PROJECT_ID`):

```tool:web_fetch
url: https://backboard.railway.com/graphql/v2
method: POST
headers: {"Authorization": "Bearer $RAILWAY_API_TOKEN", "Content-Type": "application/json"}
body: {"query": "query { serviceDomains(serviceId: \"SERVICE_ID\", environmentId: \"ENVIRONMENT_ID\", projectId: \"PROJECT_ID\") { serviceDomains { domain } customDomains { domain } } }"}
extract_mode: raw
```

---

## Error Handling

| Error | Cause | Solution |
|-------|-------|----------|
| 401 / UNAUTHENTICATED | Token is invalid or expired | Regenerate token at https://railway.com/account/tokens |
| NOT_FOUND | Project/service/environment ID doesn't exist | List projects first to get valid IDs |
| FORBIDDEN | Token lacks permission for this resource | Check token scopes or use a different token |

### Common Issues

- **Empty responses**: Make sure you're using the correct project/service/environment IDs. List projects first to discover valid IDs.
- **GraphQL errors**: Check the `errors` array in the response for details. Field names are case-sensitive.
- **Rate limiting**: Railway's API has rate limits. If you get rate-limited, wait before retrying.

---

## Typical Workflow

1. **Verify auth** — confirm token works with `me` query
2. **List projects** — discover project IDs
3. **Get project details** — find service and environment IDs
4. **Check deployments** — see current deployment status
5. **Take action** — redeploy, update env vars, etc. (confirm with user first)

---

## Best Practices

1. **Always verify auth first** before running other queries
2. **List before acting** — get IDs from list queries, don't guess
3. **Confirm mutations** — always ask the user before redeploying or changing env vars
4. **Check deployment status** after triggering a redeploy
5. **Be careful with env vars** — they may contain secrets, don't log values unnecessarily
