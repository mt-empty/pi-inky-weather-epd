# Daily forecast “date/time” semantics at **1:00am Sydney/Melbourne**

This note explains what you’ll see from:

* **BoM Weather App JSON** (`api.weather.bom.gov.au/v1/.../forecasts/daily`)
* **Open-Meteo Forecast API** (`api.open-meteo.com/v1/...`)

We focus on the common “off-by-one-day” confusion when requests happen around **local midnight** (e.g., 1:00am in Sydney/Melbourne).

---

## Scenario we’re talking about

Example request time:

* **Sydney/Melbourne local:** `2025-12-18 01:00` (AEDT, UTC+11)
* **UTC:** `2025-12-17 14:00Z`

Sydney and Melbourne behave the same for this issue because they share the same timezone rules.

---

## 1) BoM daily forecast (`/forecasts/daily`)

### What BoM returns

The BoM daily endpoint returns each day with a `date` like:

* `"date": "2020-11-06T13:00:00Z"` (example from the docs)

This is an **ISO-8601 UTC timestamp** (`Z` = UTC). The docs also show the endpoint and the field shape.

### What that timestamp *means*

In practice, that `date` value is the **start of the local forecast day**, but **expressed in UTC**.

So for **AEDT (UTC+11)**:

* Local midnight `2025-12-18 00:00 AEDT`
* is `2025-12-17 13:00Z`

That’s why you’ll see something that *looks like yesterday* in UTC (e.g., `...T13:00:00Z`) even though it corresponds to **today’s local date** once converted to `Australia/Sydney` / `Australia/Melbourne`.

This is consistent with BoM’s broader forecast database conventions: forecast times are represented in **UTC**.

### What happens when you call it at **1:00am local**

At **01:00 AEDT**, you will typically see the “today” daily entry encoded as:

* **UTC timestamp on the previous day** at **13:00Z** (during AEDT)
* **UTC timestamp on the previous day** at **14:00Z** (during AEST, UTC+10)

### Dev rule for BoM daily

✅ **Never interpret the `YYYY-MM-DD` part of BoM’s `date` as the local day.**
Instead:

1. parse `date` as UTC
2. convert to the location timezone
3. use the **converted local date** as the “forecast day key”

> Note: the BoM Weather App API also includes a warning header about restricted use; treat these endpoints accordingly.

---

## 2) Open-Meteo daily forecast (no `timezone=`)

### What Open-Meteo’s docs say

Open-Meteo’s docs state:

* `timezone` default is **GMT**
* If `timezone` is set, “all timestamps are returned as local-time and data is returned starting at **00:00 local-time**”
* If you request `daily=` variables, **parameter `timezone` is required**

### What you’ll see at **1:00am Sydney/Melbourne** if you omit timezone

If a call succeeds without `timezone=` (some wrappers/clients may behave differently), you should assume the response is **GMT-anchored**:

At **2025-12-18 01:00 AEDT** (which is **2025-12-17 14:00Z**), “today” in GMT is still **2025-12-17**, so the first daily date you’ll typically see is:

* `daily.time[0] = "2025-12-17"` (GMT day)

This looks like it’s “one day behind” from the Sydney/Melbourne perspective, because you’re already into **Dec 18 local**, but the API is using **Dec 17 GMT** as “today”.

### Fix (recommended)

Always pass one of:

* `timezone=Australia/Sydney` (or `Australia/Melbourne`)
* `timezone=auto` (let Open-Meteo resolve it from coordinates)

Then:

* daily dates align to **local midnight**
* “today” at 1:00am local will be returned as the **local calendar date** (e.g., `"2025-12-18"`).

---

## Quick comparison table (at 1:00am AEDT)

| API                  | If you don’t set timezone                                       | What “today” looks like at 1:00am AEDT                                                   |
| -------------------- | --------------------------------------------------------------- | ---------------------------------------------------------------------------------------- |
| **BoM daily**        | Always returns UTC timestamps (`...Z`)                          | “Today” local (Dec 18) appears as **Dec 17T13:00:00Z** (UTC)                             |
| **Open-Meteo daily** | Defaults to **GMT**, and docs say `timezone` required for daily | “Today” becomes **UTC/GMT day** (often appears as **Dec 17**) unless you pass a timezone |

---

## Implementation guidance (what we should standardise on)

* **Store/operate on a “local forecast date”** for UI + business logic.
* **BoM:** `local_date = convert(utc_timestamp, Australia/Sydney).date()`
* **Open-Meteo:** always include `timezone=Australia/Sydney` (or `auto`) so `daily.time[]` is already local-date aligned.

If you want, paste one real BoM JSON object + one Open-Meteo response you’re seeing at 1am and I’ll annotate exactly which entry corresponds to which local day.
