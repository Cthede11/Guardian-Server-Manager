# Guardian Desktop Application

Guardian is a professional Minecraft server management application built with modern web technologies and packaged as a cross-platform desktop application using Tauri.

## 🚀 Features

- **Real-time Server Monitoring**: Live TPS, memory usage, player count, and performance metrics
- **Server Management**: Start, stop, restart, and configure Minecraft servers
- **Player Management**: Kick, ban, and monitor players
- **World Management**: Heatmap visualization, chunk monitoring, and freeze detection
- **Pregeneration**: Queue and manage world pregeneration jobs
- **Backup System**: Automated backups with restore functionality
- **Console Access**: Real-time server console with command execution
- **Performance Optimization**: GPU acceleration and performance tuning
- **Cross-platform**: Windows, macOS, and Linux support

## 🛠️ Technology Stack

### Frontend
- **React 18** with TypeScript
- **Vite** for fast development and building
- **Tailwind CSS** for styling
- **Zustand** for state management
- **React Router** for navigation
- **Radix UI** for accessible components

### Backend
- **Rust** with Axum web framework
- **SQLite** database for configuration and data
- **WebSocket** for real-time communication
- **RCON** integration for Minecraft server communication
- **Process management** for server lifecycle

### Desktop App
- **Tauri** for cross-platform desktop packaging
- **System tray** integration
- **Native notifications**
- **File system access**
- **Auto-updater** support

## 📦 Installation

### Prerequisites

- **Node.js** 20.19.0 or higher
- **Rust** 1.77.2 or higher
- **Git**

### Development Setup

1. **Clone the repository**
   ```bash
   git clone https://github.com/guardian-team/guardian.git
   cd guardian
   ```

2. **Install dependencies**
   ```bash
   # Install frontend dependencies
   cd guardian-ui
   npm install
   
   # Install backend dependencies
   cd ../hostd
   cargo build
   ```

3. **Start development environment**
   ```bash
   # From the project root
   ./scripts/dev-desktop.ps1  # Windows PowerShell
   # or
   ./scripts/dev-desktop.sh   # Linux/macOS
   ```

### Production Build

1. **Build the desktop application**
   ```bash
   # From the project root
   ./scripts/build-desktop.ps1  # Windows PowerShell
   # or
   ./scripts/build-desktop.sh   # Linux/macOS
   ```

2. **Find the installer**
   - Windows: `guardian-ui/src-tauri/target/release/bundle/msi/Guardian_1.0.0_x64_en-US.msi`
   - macOS: `guardian-ui/src-tauri/target/release/bundle/dmg/Guardian_1.0.0_x64.dmg`
   - Linux: `guardian-ui/src-tauri/target/release/bundle/appimage/Guardian_1.0.0_x86_64.AppImage`

## 🎮 Usage

### First Launch

1. **Install the application** using the installer for your platform
2. **Launch Guardian** from your applications menu or desktop
3. **Configure your first server**:
   - Click "Add Server" in the main interface
   - Enter server details (name, Java path, server JAR path)
   - Configure JVM arguments and server settings
   - Save the configuration

### Server Management

- **Start/Stop/Restart**: Use the server controls in the main interface
- **Monitor Performance**: View real-time metrics in the Performance tab
- **Manage Players**: Use the Players tab to kick, ban, or monitor players
- **Console Access**: Send commands and view logs in the Console tab
- **World Management**: Monitor world performance and manage chunks in the World tab

### Advanced Features

- **Pregeneration**: Queue world pregeneration jobs for better performance
- **Backups**: Set up automated backups with retention policies
- **Performance Tuning**: Adjust JVM settings and performance budgets
- **GPU Acceleration**: Enable GPU-accelerated world generation

## 🔧 Configuration

### Environment Variables

- `VITE_USE_MOCK_DATA=true`: Use mock data instead of real server data (development)
- `VITE_API_URL=http://localhost:8080/api`: Backend API URL

### Backend Configuration

The backend can be configured using command-line arguments:

```bash
hostd --port 8080 --database-url sqlite:guardian.db --log-level info
```

### Server Configuration

Each server can be configured with:
- Java executable path
- Server JAR file path
- JVM arguments (memory, GC settings, etc.)
- Server arguments
- Auto-start and auto-restart settings
- RCON configuration

## 🐛 Troubleshooting

### Common Issues

1. **Backend won't start**
   - Check if port 8080 is available
   - Ensure Rust is properly installed
   - Check the console for error messages

2. **Frontend won't connect to backend**
   - Verify the backend is running on port 8080
   - Check firewall settings
   - Ensure `VITE_USE_MOCK_DATA` is set to `false`

3. **Desktop app won't launch**
   - Check if all dependencies are installed
   - Try running in development mode first
   - Check the system tray for the application

4. **Server management issues**
   - Verify Java is installed and accessible
   - Check server JAR file path
   - Ensure proper permissions for server files

### Debug Mode

Run the application in debug mode for detailed logging:

```bash
# Development mode with debug logging
npm run tauri:dev

# Or set log level in backend
hostd --log-level debug
```

## 🤝 Contributing

1. **Fork the repository**
2. **Create a feature branch**
3. **Make your changes**
4. **Add tests** for new functionality
5. **Submit a pull request**

### Development Guidelines

- Follow the existing code style
- Add TypeScript types for all new code
- Write tests for new features
- Update documentation as needed
- Test on multiple platforms

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🆘 Support

- **Documentation**: [GitHub Wiki](https://github.com/guardian-team/guardian/wiki)
- **Issues**: [GitHub Issues](https://github.com/guardian-team/guardian/issues)
- **Discussions**: [GitHub Discussions](https://github.com/guardian-team/guardian/discussions)
- **Discord**: [Guardian Discord Server](https://discord.gg/guardian)

## 🗺️ Roadmap

### Version 1.1
- [ ] Plugin system for extensions
- [ ] Advanced backup strategies
- [ ] Multi-server clustering
- [ ] Performance analytics

### Version 1.2
- [ ] Web-based remote management
- [ ] Mobile companion app
- [ ] Cloud backup integration
- [ ] Advanced automation rules

### Version 2.0
- [ ] Distributed server management
- [ ] Advanced monitoring and alerting
- [ ] Integration with popular hosting providers
- [ ] Enterprise features

---

**Guardian** - Professional Minecraft Server Management Made Simple
