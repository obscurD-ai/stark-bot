---
name: Scheduling
---

Automate tasks with cron jobs and heartbeat triggers.

## Overview

| Type | When to Use |
|------|-------------|
| **Cron Jobs** | Specific times (9 AM daily, Mondays) |
| **Heartbeat** | Regular intervals (every hour) |

---

## Cron Jobs

Run prompts on a schedule using standard cron expressions.

### Creating a Job

1. Go to **Scheduling**
2. Click **Add Job**
3. Enter name, schedule, and message
4. Save

### Cron Syntax

```
┌───────────── minute (0-59)
│ ┌───────────── hour (0-23)
│ │ ┌───────────── day of month (1-31)
│ │ │ ┌───────────── month (1-12)
│ │ │ │ ┌───────────── day of week (0-6, Sun=0)
│ │ │ │ │
* * * * *
```

### Common Patterns

| Expression | When |
|------------|------|
| `0 9 * * *` | Daily at 9:00 AM |
| `0 9 * * MON` | Mondays at 9:00 AM |
| `0 */4 * * *` | Every 4 hours |
| `30 8 * * MON-FRI` | Weekdays at 8:30 AM |
| `0 0 1 * *` | First of month, midnight |
| `*/15 * * * *` | Every 15 minutes |

### Example Jobs

**Daily Summary**
```
Schedule: 0 18 * * *
Message: Summarize today's activities and notable events
```

**Weekly Report**
```
Schedule: 0 9 * * MON
Message: Generate a weekly report and send to Discord
```

**Hourly Check**
```
Schedule: 0 * * * *
Message: Check system health and alert on issues
```

---

## Job Management

| Action | Description |
|--------|-------------|
| **Run Now** | Execute immediately |
| **Pause** | Temporarily disable |
| **Resume** | Re-enable paused job |
| **Delete** | Remove permanently |

### Execution History

Each job tracks:
- Run timestamp
- Success/failure
- Response summary

---

## Heartbeat

Simpler interval-based scheduling without cron syntax.

### Configuration

```json
{
  "enabled": true,
  "interval": "daily",
  "time": "09:00",
  "message": "Good morning! Here's your briefing.",
  "channel_id": "discord-uuid"
}
```

### Intervals

| Interval | Description |
|----------|-------------|
| Hourly | Every hour on the hour |
| Daily | Once per day at specified time |
| Weekly | Once per week on specified day |
| Custom | Custom interval in minutes |

---

## How It Works

The scheduler service:

1. Checks for due jobs every 10 seconds
2. Creates a message from the job prompt
3. Processes through the dispatcher (same as chat)
4. Records execution result
5. Calculates next run time

```
Scheduler (every 10s)
       ↓
Find due jobs
       ↓
For each job:
  → Create NormalizedMessage
  → Dispatcher processes
  → AI + tools execute
  → Log result
  → Update next_run
```

---

## With Skills

Combine scheduling with skills:

**Weather Alert**
```
Schedule: 0 7 * * *
Message: Use the weather skill for Seattle. If rain expected,
         send umbrella reminder to Telegram.
```

**PR Review Reminder**
```
Schedule: 0 10 * * MON-FRI
Message: Use github-pr skill to list PRs older than 2 days.
         Send summary to Slack if any need attention.
```

---

## Best Practices

### Clear Names
- "Daily Sales Report" not "Job 1"
- "Monday Standup" not "Weekly"

### Appropriate Intervals
- Don't schedule too frequently
- Consider timezone (jobs run in UTC)
- Avoid overlapping jobs

### Idempotent Messages
Design prompts that handle repeated execution:
- "Generate today's report" ✓
- "Append to report" ✗

### Error Handling
Include failure instructions:
```
Generate the daily report. If data unavailable,
notify team via Discord with error details.
```

---

## Timezone

- All times are **UTC**
- Dashboard shows local time
- Be explicit in job names: "Daily Report (9 AM UTC)"
