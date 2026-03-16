# Bergfex API Collector

This tool collects data from the Bergfex API for the Sonnenbühl-Genkingen ski resort and stores it in the SkiAPI database.

## Features

- **GET-only requests**: Only uses GET requests to the Bergfex API to avoid legal issues
- **Daily collection**: Designed to run once per day for up-to-date information
- **Comprehensive data**: Collects resort information, lifts, slopes, and weather data
- **Error handling**: Robust error handling and logging
- **Checkpointing**: Saves collected data to checkpoint files for debugging

## Files

- `bergfex_collector.py`: Main collector script
- `bergfex_launcher.py`: Launcher script for scheduling
- `README.md`: This documentation file

## Usage

### Manual Execution

```bash
# Basic usage
python bergfex_collector.py

# With custom resort ID
python bergfex_collector.py --resort-id sonnenbuehl-genkingen

# With custom API key
python bergfex_collector.py --api-key your-skiapi-key

# Save checkpoint data
python bergfex_collector.py --checkpoint
```

### Using the Launcher

```bash
# Basic usage
python bergfex_launcher.py

# With custom parameters
python bergfex_launcher.py --resort-id sonnenbuehl-genkingen --api-key your-skiapi-key
```

## API Configuration

### SkiAPI Configuration

The collector needs access to your SkiAPI instance:

1. **Environment Variable**: Set `SKI_API_KEY` environment variable
2. **API Key File**: Create `api_key.env` file in the project root with your API key
3. **Command Line**: Use `--api-key` parameter

### Bergfex API

**Important**: The Bergfex API may require authentication or have different access requirements. If you encounter 404 errors:

1. **Check API Access**: Verify you have proper access to the Bergfex API
2. **Authentication**: Some endpoints may require API keys or tokens
3. **Resort ID**: The correct resort ID format may differ from expected patterns
4. **Documentation**: Refer to the official Bergfex API documentation

The collector uses these endpoints (subject to access requirements):
- `/public/skiresorts/{resort_id}` - Resort information
- `/public/skiresorts/{resort_id}/lifts` - Lift information  
- `/public/skiresorts/{resort_id}/slopes` - Slope information
- `/public/skiresorts/{resort_id}/weather` - Weather information

### Testing API Access

Use the provided test script to verify API access:

```bash
python test_bergfex_api.py
```

This will help identify:
- Correct resort ID format
- Authentication requirements
- Available endpoints

### Authentication

The Bergfex API requires authentication. If you have an API token:

```bash
# Test with authentication
python bergfex_collector.py --resort-id sonnenbuehl-genkingen --api-key YOUR_BERGFEX_TOKEN

# Or set as environment variable
export BERGFEX_API_TOKEN=your_token_here
python bergfex_collector.py --resort-id sonnenbuehl-genkingen
```

**Note**: The collector currently uses SkiAPI authentication for your local API. For Bergfex API access, you'll need to obtain an API token from Bergfex and modify the collector to include it in requests.

## Data Mapping

### Lift Types

| Bergfex Type | SkiAPI Type |
|-------------|-------------|
| gondola | gondola |
| cable_car | cable_car |
| chair_lift | chairlift |
| mixed_lift | chairlift |
| t-bar | draglift |
| j-bar | draglift |
| platter | draglift |
| rope_tow | draglift |
| magic_carpet | magic_carpet |
| surface_lift | draglift |
| ski_lift | chairlift |

### Slope Difficulties

| Bergfex Difficulty | SkiAPI Difficulty |
|-------------------|-------------------|
| novice | green |
| easy | blue |
| intermediate | red |
| advanced | black |
| expert | black |
| beginner | green |
| family | blue |

### Status Mapping

| Bergfex Status | SkiAPI Status |
|---------------|---------------|
| open | open |
| closed | closed |
| maintenance | maintenance |
| unknown | unknown |

## Scheduling

### Linux/macOS (cron)

Add to your crontab (`crontab -e`):

```bash
# Run daily at 6 AM
0 6 * * * /usr/bin/python3 /path/to/Ski_API/scripts/data_tools/api_collector_tools/bergfex_launcher.py >> /path/to/logs/bergfex_cron.log 2>&1
```

### Windows (Task Scheduler)

1. Open Task Scheduler
2. Create Basic Task
3. Set trigger to "Daily" at your preferred time
4. Set action to "Start a program"
5. Program: `python`
6. Arguments: `C:\path\to\Ski_API\scripts\data_tools\api_collector_tools\bergfex_launcher.py`
7. Start in: `C:\path\to\Ski_API`

## Logging

Logs are saved to:
- `logs/api_collector/bergfex_collector_YYYY-MM-DD_HH-MM-SS.log` - Collection logs
- `logs/api_collector/bergfex_launcher.log` - Launcher logs

## Checkpointing

When using the `--checkpoint` flag, collected data is saved to:
- `checkpoints/bergfex/` directory

This is useful for debugging and data analysis.

## Error Handling

The collector includes comprehensive error handling:

- **Network errors**: Retries failed requests
- **API errors**: Logs and continues with available data
- **Data validation**: Validates and normalizes all collected data
- **Missing data**: Handles missing fields gracefully

## Dependencies

Required Python packages:
- `aiohttp` - Async HTTP client for Bergfex API
- `requests` - HTTP client for SkiAPI
- `logging` - Built-in logging
- `argparse` - Command line argument parsing
- `asyncio` - Async programming support

Install dependencies:
```bash
pip install aiohttp requests
```

## Legal Compliance

This tool strictly follows the requirement to use only GET requests to the Bergfex API. No POST, PUT, DELETE, or other request types are used to ensure compliance with Bergfex's terms of service.

## Troubleshooting

### Common Issues

1. **API Key Errors**: Ensure your SkiAPI key is properly configured
2. **Network Errors**: Check internet connection and Bergfex API availability
3. **Permission Errors**: Ensure write permissions for log and checkpoint directories
4. **Python Path**: Ensure Python is in your PATH or use full path

### Debug Mode

For detailed debugging, check the log files in `logs/api_collector/` directory.

### Manual Testing

Test the collector manually before scheduling:

```bash
python bergfex_collector.py --resort-id sonnenbuehl-genkingen --checkpoint
```

This will save all collected data to checkpoint files for inspection.