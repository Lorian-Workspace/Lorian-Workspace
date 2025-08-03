# 🚀 Guía de Releases Automáticas

Este proyecto está configurado para **releases completamente automáticas**. Solo necesitas hacer commits y GitHub Actions se encarga del resto.

## 📝 Cómo Funciona

### 1. **Tipos de Commit y Versiones**

El sistema detecta automáticamente qué tipo de versión crear basándose en tus mensajes de commit:

#### 🔴 **Major Version** (1.0.0 → 2.0.0)
Para cambios que rompen compatibilidad:
```bash
git commit -m "BREAKING CHANGE: nueva API completamente diferente"
git commit -m "feat!: cambio radical en la configuración"
```

#### 🟡 **Minor Version** (1.0.0 → 1.1.0)
Para nuevas características:
```bash
git commit -m "feat: agregado soporte para múltiples perfiles"
git commit -m "feature: nuevo sistema de notificaciones"
```

#### 🟢 **Patch Version** (1.0.0 → 1.0.1)
Para correcciones y mejoras pequeñas:
```bash
git commit -m "fix: corregido error en los botones de Discord"
git commit -m "bugfix: solucionado problema de conexión"
git commit -m "chore: actualizada documentación"
git commit -m "docs: mejorado README"
git commit -m "style: formato de código"
git commit -m "refactor: reorganizado código"
```

### 2. **Proceso Automático**

Cuando haces `git push` al branch `main`:

1. ✅ **Análisis**: GitHub Actions analiza tus commits
2. 🔍 **Detección**: Determina si necesita crear una nueva versión
3. 📦 **Compilación**: Compila la aplicación para Windows
4. 🏷️ **Tag**: Crea automáticamente el tag de versión
5. 📋 **Release**: Crea la release con notas automáticas
6. ⬆️ **Upload**: Sube el archivo ZIP listo para descargar

## 🎯 Ejemplos Prácticos

### Ejemplo 1: Corregir Bugs
```bash
# Tu workflow normal
git add .
git commit -m "fix: corregidos botones de Discord Rich Presence"
git push

# ✨ Resultado: Nueva versión 1.0.1 creada automáticamente
```

### Ejemplo 2: Nueva Funcionalidad
```bash
# Agregar nueva característica
git add .
git commit -m "feat: agregado soporte para temas oscuros"
git push

# ✨ Resultado: Nueva versión 1.1.0 creada automáticamente
```

### Ejemplo 3: Cambio Mayor
```bash
# Cambio que rompe compatibilidad
git add .
git commit -m "BREAKING CHANGE: nueva estructura de configuración"
git push

# ✨ Resultado: Nueva versión 2.0.0 creada automáticamente
```

## 📁 Qué se Incluye en cada Release

Cada release automática incluye:
- ✅ **Ejecutable compilado** (`lorianworkspace.exe`)
- ✅ **Documentación** (README, LICENSE, CHANGELOG)
- ✅ **Icono** de la aplicación
- ✅ **Archivo de versión** con información del build
- ✅ **Notas de release** generadas automáticamente

## 🔧 Configuración Avanzada

### Si NO Quieres Release Automática
Si haces commits pero NO quieres que se cree una nueva versión, usa estos prefijos:
```bash
git commit -m "temp: trabajo en progreso"
git commit -m "wip: mejorando algoritmo"
git commit -m "backup: guardando cambios"
```

### Forzar un Tipo de Versión Específico
```bash
# Forzar patch aunque sea una feature
git commit -m "fix: nueva característica menor"

# Forzar major para cambio importante
git commit -m "BREAKING CHANGE: cualquier mensaje"
```

## 📊 Ver el Estado

Puedes ver el estado de tus releases automáticas en:
- **GitHub Actions**: Ve a la pestaña "Actions" en tu repo
- **Releases**: Ve a la pestaña "Releases" para ver todas las versiones
- **Tags**: Ve los tags creados automáticamente

## 🚨 Solución de Problemas

### Si el Workflow Falla
1. Ve a **GitHub Actions** y revisa los logs
2. Los errores más comunes son:
   - Problemas de compilación → Revisa tu código Rust
   - Permisos → Ya están configurados automáticamente
   - Formato de commit → Revisa que uses los prefijos correctos

### Si No se Crea Release
Verifica que tu commit tenga uno de estos prefijos:
- `feat:`, `fix:`, `chore:`, `docs:`, `style:`, `refactor:`, `test:`
- O palabras clave: `BREAKING CHANGE`, `feature`, `bugfix`

## 💡 Tips

1. **Combina varios cambios** en un solo commit si son relacionados
2. **Usa mensajes descriptivos** para mejores notas de release
3. **Revisa las releases automáticas** para asegurarte de que todo esté correcto
4. **El primer push** creará la versión v1.0.0 si no hay tags previos

---

**¡Eso es todo!** 🎉 Ahora solo necesitas hacer commits normales y el sistema se encarga de todo automáticamente.