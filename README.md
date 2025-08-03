# Lorian Workspace

[![CI](https://github.com/tu-usuario/Lorian-Workspace/workflows/CI/badge.svg)](https://github.com/tu-usuario/Lorian-Workspace/actions/workflows/ci.yml)
[![Auto Release](https://github.com/tu-usuario/Lorian-Workspace/workflows/Auto%20Release/badge.svg)](https://github.com/tu-usuario/Lorian-Workspace/actions/workflows/auto-release.yml)
[![Release](https://github.com/tu-usuario/Lorian-Workspace/workflows/Release/badge.svg)](https://github.com/tu-usuario/Lorian-Workspace/actions/workflows/release.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-stable-brightgreen.svg)](https://www.rust-lang.org)
[![Windows](https://img.shields.io/badge/platform-Windows-blue.svg)](https://www.microsoft.com/windows)

**A comprehensive background application suite for content creators and developers.**

## 🚀 Features

### 🎮 Background Custom Discord RPC
- **Automatic activity rotation** with customizable intervals
- **Rich presence display** with custom images, buttons, and status messages  
- **Hot reload configuration** - edit settings without restarting
- **System tray integration** for seamless background operation
- **No console window** - truly invisible background application
- **Intelligent reconnection** handling for Discord client restarts

### ⚙️ System Integration
- **Windows system tray icon** with right-click context menu
- **Automatic startup** configuration support
- **AppData storage** for user settings and logs
- **Real-time configuration watching** and reloading

## 📦 Installation

### Requirements
- Windows 10/11
- Discord Desktop Application
- Discord Developer Account (for custom application setup)

### Quick Start
1. Download the latest release from [Releases](../../releases)
2. Extract `lorianworkspace.exe` to your desired location
3. Run the application - it will create default configuration files
4. Configure your Discord Application ID and activities
5. Enjoy automated Discord Rich Presence!

## 🤖 Automatic Releases

This project features **fully automated releases** - no manual intervention needed!

### How it Works
- 🚀 **Push commits** to main branch
- 🔍 **Automatic detection** of version type based on commit messages
- 📦 **Automatic compilation** for Windows
- 🏷️ **Automatic tagging** and release creation
- ⬇️ **Ready-to-download** ZIP files with executables

### Commit Message Conventions
- `feat:` → Minor version bump (new features)
- `fix:` → Patch version bump (bug fixes)  
- `BREAKING CHANGE:` → Major version bump (breaking changes)
- `chore:`, `docs:`, `style:` → Patch version bump

### Example
```bash
git commit -m "feat: added dark theme support"
git push
# ✨ Automatically creates v1.1.0 release!
```

📖 **Detailed guide**: See [AUTO_RELEASE_GUIDE.md](AUTO_RELEASE_GUIDE.md) for complete instructions.

## 📋 Logging System

Lorian Workspace includes an intelligent logging system:

### Features
- **📝 Automatic logging** to `%APPDATA%/lorianworkspace/app.log`
- **🔄 Auto-rotation** when log file exceeds 5MB (old logs are deleted)
- **📊 Size monitoring** displayed on startup
- **🚫 No backup files** - keeps storage clean and minimal

### Log Information
- All Discord Rich Presence activities and errors
- Connection status and reconnection attempts  
- Configuration changes and hot reloads
- Button configuration details and debugging info
- System tray interactions and commands

### Example Log Output
```
[2024-01-15 10:30:15] 🚀 Iniciando Lorian Workspace...
[2024-01-15 10:30:15] 📝 Logs guardados en: C:\Users\User\AppData\Roaming\lorianworkspace\app.log
[2024-01-15 10:30:15] 🔄 Rotación automática: logs se borran al superar 5MB
[2024-01-15 10:30:15] 📊 Tamaño actual del log: 2.34MB
[2024-01-15 10:30:16] ✅ Discord RPC conectado exitosamente!
[2024-01-15 10:30:16] 🔘 Configurando 2 botones para la actividad
```

## 🔧 Configuration

The application automatically creates configuration files in `%APPDATA%/lorianworkspace/`:

- `config.json` - Main configuration file
- `app.log` - Application logs (auto-rotated when >5MB)

### Setting up Discord Application

1. Visit [Discord Developer Portal](https://discord.com/developers/applications)
2. Create a new application or use an existing one
3. Copy the **Application ID** from the General Information page
4. Upload custom images in the **Rich Presence > Art Assets** section
5. Update your `config.json` with your Application ID and image names

### Example Configuration

```json
{
  "discord": {
    "app_id": "YOUR_APPLICATION_ID_HERE",
    "activities": [
      {
        "name": "working",
        "details": "🎨 Creating amazing content",
        "state": "In the zone",
        "large_image": "main_logo",
        "large_text": "Lorian Workspace",
        "small_image": "status_online",
        "small_text": "Online",
        "duration_seconds": 30,
        "buttons": [
          {
            "label": "🌐 Visit Website",
            "url": "https://example.com"
          }
        ]
      }
    ]
  }
}
```

## 🎯 Usage

### System Tray Controls
- **Right Click** → Open context menu with all options
- **Double Click** → Show/hide console for debugging
- **Left Click** → Display current status

### Available Commands
- **Pause/Resume** - Stop or start activity rotation
- **Next Activity** - Manually switch to next activity
- **Reload Config** - Apply configuration changes instantly  
- **Show Status** - Display current application status
- **Open Config** - Edit configuration file
- **Exit** - Close application

### Automatic Features
- **Activity Rotation** - Cycles through configured activities automatically
- **Auto-Reconnection** - Handles Discord restarts gracefully
- **Configuration Watching** - Automatically reloads when config.json changes
- **Error Recovery** - Continues running even if Discord disconnects

## 🛠️ Development

### Building from Source

```bash
# Clone the repository
git clone https://github.com/tu-usuario/Lorian-Workspace.git
cd Lorian-Workspace

# Build for Windows
cargo build --release --target x86_64-pc-windows-gnu

# The executable will be in target/x86_64-pc-windows-gnu/release/lorianworkspace.exe
```

### Project Structure
```
Lorian-Workspace/
├── .github/             # GitHub workflows and templates
│   ├── workflows/       # CI/CD workflows
│   ├── ISSUE_TEMPLATE/  # Issue templates
│   └── SECURITY.md      # Security policy
├── src/
│   └── main.rs          # Main application code
├── build.rs             # Build script for Windows resources
├── icon.ico             # Application icon
├── icon.rc              # Windows resource file
├── Cargo.toml           # Dependencies and build configuration
├── LICENSE              # MIT License
├── CONTRIBUTING.md      # Contributing guidelines
├── CODE_OF_CONDUCT.md   # Code of conduct
├── CHANGELOG.md         # Project changelog
└── README.md            # This file
```

## 📋 Troubleshooting

### Common Issues

**Application doesn't appear in system tray**
- Check if the application is running in Task Manager
- Restart the application as administrator
- Check Windows system tray settings

**Discord activities not showing**
- Verify Discord is running and logged in
- Confirm Application ID is correct in config.json
- Check that uploaded images match the names in configuration
- Review logs in `%APPDATA%/lorianworkspace/app.log`

**Configuration not reloading**
- Ensure config.json has valid JSON syntax
- Check file permissions in AppData folder
- Restart the application if hot reload fails

### Logs Location
Application logs are stored in: `%APPDATA%/lorianworkspace/app.log`

## 📄 License

MIT License

Copyright (c) 2024 Lorian Workspace

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.

## 🤝 Contributing

Contributions are welcome! Please read our [Contributing Guidelines](CONTRIBUTING.md) for details on our code of conduct and the process for submitting pull requests.

### Quick Contributing Guide
1. Fork the repository
2. Create your feature branch (`git checkout -b feature/AmazingFeature`)
3. Commit your changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

See our [Code of Conduct](CODE_OF_CONDUCT.md) for community guidelines.

## 📞 Support

For support and questions:
- 📖 Check the troubleshooting section above
- 📋 Review application logs in AppData
- 🐛 [Report bugs](https://github.com/tu-usuario/Lorian-Workspace/issues/new?template=bug_report.md)
- 💡 [Request features](https://github.com/tu-usuario/Lorian-Workspace/issues/new?template=feature_request.md)
- 💬 [Join discussions](https://github.com/tu-usuario/Lorian-Workspace/discussions)
- 🔒 [Report security issues](https://github.com/tu-usuario/Lorian-Workspace/security/advisories/new)

---

**Made with ❤️ by the Lorian Workspace team**