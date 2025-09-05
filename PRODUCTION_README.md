# Guardian - Professional Minecraft Server Manager

## 🚀 Production Release

Guardian is a professional Minecraft server management application that combines advanced monitoring, automation, and optimization features into a single, easy-to-use desktop application.

## ✨ Features

- **Real-time Server Monitoring**: Live TPS, memory usage, and player statistics
- **Advanced Performance Optimization**: GPU-accelerated chunk generation and performance tuning
- **Automated Backup Management**: Scheduled backups with multiple strategies
- **Professional Server Management**: Start, stop, restart, and configure servers
- **Real-time Console Access**: Direct server command execution and log monitoring
- **Player Management**: Kick, ban, teleport, and manage players
- **World Analysis**: Heatmap visualization and freeze detection
- **Mod Management**: Conflict detection and compatibility rules
- **Performance Metrics**: Detailed performance analysis and recommendations

## 🎯 Quick Start

### Installation

1. **Download** the Guardian installer from the releases page
2. **Run** the MSI installer and follow the setup wizard
3. **Launch** Guardian from your Start Menu or Desktop
4. **Configure** your first Minecraft server in the Settings tab

### First Time Setup

1. Open Guardian
2. Navigate to **Settings** → **Server Configuration**
3. Set your Minecraft server directory
4. Configure server settings (memory, port, etc.)
5. Create your first server
6. Start managing!

## 🔧 System Requirements

### Minimum Requirements
- **OS**: Windows 10/11 (64-bit)
- **RAM**: 4GB available memory
- **Storage**: 2GB free disk space
- **Network**: Internet connection for updates

### Recommended Requirements
- **OS**: Windows 11 (64-bit)
- **RAM**: 8GB+ available memory
- **Storage**: 10GB+ free disk space
- **GPU**: DirectX 11 compatible (for GPU acceleration)
- **Network**: Stable internet connection

## 📁 File Structure

```
Guardian/
├── Guardian.exe              # Main application
├── hostd.exe                 # Backend service
├── gpu-worker.exe           # GPU acceleration service
├── configs/                  # Configuration files
│   ├── server.yaml          # Server configuration
│   ├── rules.yaml           # Mod compatibility rules
│   └── hostd.yaml           # Backend configuration
├── data/                     # Application data
│   ├── guardian.db          # SQLite database
│   └── gpu-cache/           # GPU cache files
└── logs/                     # Application logs
```

## 🎮 Supported Minecraft Versions

- **Vanilla**: 1.16.5 - Latest
- **Forge**: 1.16.5 - Latest
- **Fabric**: 1.16.5 - Latest
- **Paper**: 1.16.5 - Latest
- **Spigot**: 1.16.5 - Latest

## 🔒 Security Features

- **Secure Authentication**: Built-in user management
- **Data Encryption**: Sensitive data is encrypted at rest
- **Network Security**: All communications use secure protocols
- **Access Control**: Role-based permissions system

## 📊 Performance Features

- **GPU Acceleration**: Hardware-accelerated chunk generation
- **Memory Optimization**: Intelligent memory management
- **CPU Optimization**: Multi-threaded processing
- **Network Optimization**: Efficient data transfer protocols

## 🛠️ Troubleshooting

### Common Issues

**Q: Guardian won't start**
- Check Windows Defender/antivirus isn't blocking the application
- Ensure you have administrator privileges
- Verify all required files are present

**Q: Backend services not starting**
- Check if ports 8080 and 25565 are available
- Ensure no other Minecraft servers are running
- Check the logs folder for error messages

**Q: GPU acceleration not working**
- Verify your GPU supports DirectX 11
- Update your graphics drivers
- Check Windows compatibility settings

**Q: Can't connect to servers**
- Verify server configuration is correct
- Check firewall settings
- Ensure server files are accessible

### Getting Help

1. **Check the logs** in the `logs/` folder
2. **Review the documentation** in the Help section
3. **Contact support** through the application
4. **Visit our website** for additional resources

## 🔄 Updates

Guardian automatically checks for updates when you start the application. You can also manually check for updates in the Settings menu.

### Update Process

1. Download the latest installer
2. Run the installer (it will update the existing installation)
3. Restart Guardian
4. Your settings and data will be preserved

## 📋 Configuration

### Server Configuration

```yaml
# Example server configuration
server:
  name: "My Minecraft Server"
  version: "1.20.1"
  loader: "vanilla"
  memory: 4096
  port: 25565
  world: "world"
  mods: "mods"
```

### Performance Settings

```yaml
# Performance configuration
performance:
  enable_gpu_acceleration: true
  max_concurrent_operations: 5
  cache_size: 100
  memory_threshold: 0.8
```

## 🎯 Best Practices

### Server Management
- **Regular Backups**: Set up automated daily backups
- **Monitor Performance**: Keep an eye on TPS and memory usage
- **Update Regularly**: Keep your server and mods updated
- **Resource Management**: Don't overload your system

### Security
- **Strong Passwords**: Use complex passwords for all accounts
- **Regular Updates**: Keep Guardian and your system updated
- **Backup Data**: Regularly backup your server data
- **Monitor Access**: Review user access logs regularly

## 📞 Support

### Technical Support
- **Email**: support@guardian-mc.com
- **Discord**: https://discord.gg/guardian-mc
- **Documentation**: https://docs.guardian-mc.com

### Community
- **Forums**: https://forums.guardian-mc.com
- **GitHub**: https://github.com/guardian-team/guardian
- **Reddit**: https://reddit.com/r/guardian-mc

## 📄 License

Guardian is licensed under the MIT License. See the LICENSE file for details.

## 🙏 Acknowledgments

- **Minecraft Community** for inspiration and feedback
- **Open Source Contributors** for their valuable contributions
- **Beta Testers** for helping us refine the application

---

**Guardian Team** - Professional Minecraft Server Management

*Copyright © 2024 Guardian Team. All rights reserved.*
