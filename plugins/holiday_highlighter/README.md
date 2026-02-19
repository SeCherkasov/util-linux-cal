# Holiday Highlighter Plugin

Plugin for `cal` that adds holiday highlighting via isdayoff.ru API.

## Features

- **Automatic country detection** from system locale
- **Data caching** to reduce API requests
- **Multiple country support**: Russia, Belarus, Kazakhstan, USA, Uzbekistan, Turkey, Latvia
- **Day type data**: working days, weekends, shortened days, official holidays

## Building

```bash
# Build entire workspace
cargo build --release --workspace

# Plugin only
cargo build --release -p holiday_highlighter
```

## Installation

### User-local

```bash
mkdir -p ~/.local/lib/cal/plugins
cp target/release/libholiday_highlighter.so ~/.local/lib/cal/plugins/
```

### System-wide

```bash
sudo mkdir -p /usr/lib/cal/plugins
sudo cp target/release/libholiday_highlighter.so /usr/lib/cal/plugins/
```

## Usage

```bash
# Highlight holidays for current country
cal -H

# Year with holidays
cal -y -H

# Three months with holidays
cal -3 -H
```

## Supported countries

| Code | Country | Auto-detect locales |
|------|---------|---------------------|
| RU | Russia | ru_RU, ru_BY, ru_KZ, ru_UZ, ru_LV |
| BY | Belarus | be_BY, ru_BY |
| KZ | Kazakhstan | kk_KZ, ru_KZ |
| US | USA | en_US, en |
| UZ | Uzbekistan | uz_UZ, ru_UZ |
| TR | Turkey | tr_TR |
| LV | Latvia | lv_LV, ru_LV |

## API

The plugin uses [isdayoff.ru API](https://isdayoff.ru/):

### Monthly request

```
GET https://isdayoff.ru/api/getdata?year=2026&month=01&pre=1
```

### Yearly request

```
GET https://isdayoff.ru/api/getdata?year=2026&pre=1
```

Parameter `pre=1` includes pre-holiday shortened days information.

## Data format

Each character in the response represents a day of the month:

| Character | Day type | Description |
|-----------|----------|-------------|
| `0` | Working | Regular working day |
| `1` | Weekend | Saturday or Sunday |
| `2` | Shortened | Pre-holiday shortened day |
| `8` | Holiday | Official public holiday |

## Environment variables

| Variable | Description |
|----------|-------------|
| `LC_ALL` | Priority locale for country detection |
| `LC_TIME` | Locale for country detection |
| `LANG` | Fallback locale for country detection |
