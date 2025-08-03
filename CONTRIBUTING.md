# Contributing to Lorian Workspace

¡Gracias por tu interés en contribuir a Lorian Workspace! 🎉

## 📋 Cómo Contribuir

### 🐛 Reportar Bugs

Si encuentras un bug, por favor:

1. Verifica que no exista ya un issue similar
2. Crea un nuevo issue con:
   - Descripción clara del problema
   - Pasos para reproducir el bug
   - Comportamiento esperado vs actual
   - Información del sistema (Windows version, Discord version)
   - Logs relevantes de `%APPDATA%/lorianworkspace/app.log`

### 💡 Sugerir Mejoras

Para nuevas características:

1. Abre un issue de "Feature Request"
2. Describe la funcionalidad deseada
3. Explica por qué sería útil
4. Proporciona ejemplos de uso si es posible

### 🔧 Desarrollo

#### Configuración del Entorno

1. **Instalar Rust**:
   ```bash
   # Instalar rustup (si no lo tienes)
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   
   # Instalar target para Windows
   rustup target add x86_64-pc-windows-gnu
   ```

2. **Clonar el repositorio**:
   ```bash
   git clone https://github.com/tu-usuario/Lorian-Workspace.git
   cd Lorian-Workspace
   ```

3. **Instalar dependencias**:
   ```bash
   cargo build
   ```

#### Proceso de Desarrollo

1. **Fork** el repositorio
2. **Crea** una rama para tu feature: `git checkout -b feature/nueva-funcionalidad`
3. **Desarrolla** tu código siguiendo las convenciones
4. **Testea** tus cambios: `cargo test`
5. **Formatea** el código: `cargo fmt`
6. **Revisa** con Clippy: `cargo clippy`
7. **Commit** tus cambios con mensajes descriptivos
8. **Push** a tu rama: `git push origin feature/nueva-funcionalidad`
9. **Abre** un Pull Request

#### Convenciones de Código

- **Formato**: Usa `cargo fmt` antes de commit
- **Linting**: Asegúrate de que `cargo clippy` no muestre warnings
- **Tests**: Añade tests para nuevas funcionalidades
- **Comentarios**: Documenta código complejo en español o inglés
- **Commits**: Usa mensajes descriptivos en español

#### Estructura de Commits

```
tipo(scope): descripción corta

Descripción más detallada si es necesaria.

- Cambio específico 1
- Cambio específico 2

Fixes #123
```

Tipos de commit:
- `feat`: Nueva funcionalidad
- `fix`: Corrección de bugs
- `docs`: Cambios en documentación
- `style`: Cambios de formato (no afectan código)
- `refactor`: Refactoring de código
- `test`: Añadir o modificar tests
- `chore`: Tareas de mantenimiento

### 🧪 Testing

```bash
# Ejecutar todos los tests
cargo test

# Ejecutar tests con output detallado
cargo test -- --nocapture

# Ejecutar un test específico
cargo test nombre_del_test
```

### 📝 Documentación

- Actualiza el README.md si añades nuevas características
- Documenta funciones públicas con comentarios de documentación
- Actualiza ejemplos de configuración si es necesario

### 🔄 Pull Request Guidelines

#### Antes de enviar:

- [ ] El código compila sin warnings
- [ ] Todos los tests pasan
- [ ] Código formateado con `cargo fmt`
- [ ] Sin warnings de `cargo clippy`
- [ ] Documentación actualizada
- [ ] Changelog actualizado (si aplica)

#### En el Pull Request:

- **Título descriptivo** del cambio
- **Descripción detallada** de qué hace el PR
- **Screenshots** si hay cambios visuales
- **Referencia** a issues relacionados
- **Lista de cambios** realizados

### 🚀 Releases

Los releases siguen [Semantic Versioning](https://semver.org/):

- **MAJOR** (v2.0.0): Cambios incompatibles
- **MINOR** (v1.1.0): Nuevas funcionalidades compatibles
- **PATCH** (v1.0.1): Correcciones de bugs

### 📞 Obtener Ayuda

Si tienes preguntas:

1. Revisa la documentación existente
2. Busca en issues cerrados
3. Abre un issue con la etiqueta "question"
4. Únete a las discusiones del repositorio

### 🎯 Áreas de Contribución

Especialmente buscamos ayuda en:

- **UI/UX**: Mejoras en la interfaz del system tray
- **Performance**: Optimizaciones de rendimiento
- **Features**: Nuevas funcionalidades para Discord RPC
- **Cross-platform**: Soporte para Linux/macOS
- **Documentation**: Mejoras en documentación
- **Testing**: Más cobertura de tests

### 📄 Licencia

Al contribuir, aceptas que tus contribuciones serán licenciadas bajo la licencia MIT del proyecto.

---

¡Gracias por ayudar a hacer Lorian Workspace mejor! 🚀