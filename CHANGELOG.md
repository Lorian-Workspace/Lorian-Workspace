# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- GitHub workflow for CI/CD
- Comprehensive project documentation
- Issue and PR templates
- Security policy
- Contributing guidelines
- Code of conduct

## [0.1.0] - 2024-XX-XX

### Added
- 🎮 Background Custom Discord RPC with automatic activity rotation
- ⚙️ Windows system tray integration with context menu
- 🔄 Hot reload configuration support
- 📁 AppData storage for user settings and logs
- 🛡️ Intelligent reconnection handling for Discord client restarts
- 🖼️ Rich presence display with custom images, buttons, and status messages
- ⏱️ Customizable activity intervals
- 🔧 Real-time configuration watching and reloading
- 🚫 No console window - truly invisible background application
- 🔄 Automatic startup configuration support
- 📋 Comprehensive logging system
- ⚡ Performance optimizations for background operation

### Technical Details
- Built with Rust for performance and reliability
- Uses tokio for async operations
- Discord Rich Presence integration
- Windows-native system tray implementation
- File system watching for configuration changes
- Cross-thread communication with channels

### Configuration
- JSON-based configuration system
- Support for multiple activities with rotation
- Customizable Discord application settings
- Activity duration configuration
- Button and image customization

### Dependencies
- tokio: Async runtime
- discord-rich-presence: Discord RPC client
- serde: JSON serialization
- notify: File system watching
- windows: Windows API bindings

[Unreleased]: https://github.com/tu-usuario/Lorian-Workspace/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/tu-usuario/Lorian-Workspace/releases/tag/v0.1.0