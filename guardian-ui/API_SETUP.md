# API Setup Instructions

This application integrates with CurseForge and Modrinth APIs to provide real mod and modpack data. Follow these instructions to set up the required API keys.

## Required API Keys

### 1. CurseForge API Key (Required)

1. Go to the [CurseForge Developer Portal](https://docs.curseforge.com/#authentication)
2. Sign in with your CurseForge account
3. Create a new API key
4. Copy the API key

### 2. Modrinth API Key (Optional)

1. Go to [Modrinth API](https://modrinth.com/api)
2. Sign in with your Modrinth account
3. Generate an API key (optional, but recommended for higher rate limits)
4. Copy the API key

## Configuration

### Option 1: Environment Variables (Recommended)

Create a `.env` file in the `guardian-ui` directory:

```env
# API Keys
VITE_CURSEFORGE_API_KEY=your_curseforge_api_key_here
VITE_MODRINTH_API_KEY=your_modrinth_api_key_here

# Backend API URL
VITE_API_BASE_URL=http://localhost:8080
```

### Option 2: Direct Configuration

Edit `src/lib/config/api-keys.ts` and replace the placeholder values:

```typescript
export const API_KEYS = {
  CURSEFORGE: 'your_actual_curseforge_api_key',
  MODRINTH: 'your_actual_modrinth_api_key', // Optional
};
```

## API Features

### CurseForge Integration
- Search and browse mods
- Get mod details and versions
- Download mod files
- Minecraft version compatibility

### Modrinth Integration
- Search and browse mods
- Get mod details and versions
- Download mod files
- Minecraft version compatibility
- Modern mod loader support (Fabric, Quilt, NeoForge)

### Unified Search
- Combines results from both platforms
- Deduplicates mods by name and version
- Sorts by popularity, downloads, or name
- Handles API failures gracefully

## Rate Limits

- **CurseForge**: 100 requests per minute (with API key)
- **Modrinth**: 300 requests per minute (with API key), 60 without

The application includes built-in rate limiting and retry logic to handle these limits gracefully.

## Troubleshooting

### Common Issues

1. **"API key invalid" error**
   - Verify your API key is correct
   - Check that the key is properly set in environment variables

2. **"Rate limit exceeded" error**
   - The application will automatically retry after a delay
   - Consider getting a Modrinth API key for higher limits

3. **"Network error" or "Failed to fetch"**
   - Check your internet connection
   - Verify the API endpoints are accessible
   - Check browser console for CORS errors

### Debug Mode

Enable debug logging by opening browser developer tools and looking for console messages starting with:
- `Searching mods from...`
- `Found X mods from...`
- `Error searching mods from...`

## Security Notes

- Never commit API keys to version control
- Use environment variables for production deployments
- Rotate API keys regularly
- Monitor API usage to avoid exceeding limits
