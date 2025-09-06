# API Setup Guide

This guide will help you set up API keys for CurseForge and Modrinth to enable full functionality in Guardian.

## CurseForge API Setup

### 1. Get Your API Key
1. Visit the [CurseForge Developer Portal](https://docs.curseforge.com/#authentication)
2. Sign in with your CurseForge account
3. Navigate to the API section
4. Generate a new API key
5. Copy the API key for use in Guardian

### 2. Configure in Guardian
1. Open Guardian Desktop Application
2. Go to Settings → API Keys
3. Paste your CurseForge API key
4. Click "Test API Key" to verify
5. Click "Save API Keys"

## Modrinth API Setup (Optional)

### 1. Get Your API Token
1. Visit [Modrinth Settings](https://modrinth.com/settings/tokens)
2. Sign in with your Modrinth account
3. Navigate to the "API Tokens" section
4. Create a new personal access token
5. Copy the token for use in Guardian

### 2. Configure in Guardian
1. Open Guardian Desktop Application
2. Go to Settings → API Keys
3. Paste your Modrinth API token
4. Click "Test API Key" to verify
5. Click "Save API Keys"

## Rate Limits

- **CurseForge**: 100 requests per minute
- **Modrinth**: 300 requests per minute (same limit with or without token)

## Compliance

Guardian fully complies with:
- [CurseForge 3rd Party API Terms and Conditions](https://support.curseforge.com/en/support/solutions/articles/9000207405-curse-forge-3rd-party-api-terms-and-conditions)
- Modrinth API Terms of Service

## Features Enabled

With API keys configured, you'll have access to:
- Real-time mod browsing and search
- Mod compatibility information
- Direct download links to official platforms
- Comprehensive mod database
- Smart filtering and categorization

## Troubleshooting

### API Key Not Working
- Verify the key is copied correctly
- Check that the key is active and not expired
- Ensure you have proper permissions

### Rate Limit Exceeded
- Wait for the rate limit window to reset
- Consider upgrading to a higher tier if available
- Optimize your usage patterns

### No Mods Showing
- Verify API keys are configured correctly
- Check your internet connection
- Ensure the mods exist on the respective platforms

## Support

If you encounter issues:
1. Check the error messages in the Guardian application
2. Verify your API keys are correct
3. Contact support through the GitHub repository
4. Check the official API documentation for updates
