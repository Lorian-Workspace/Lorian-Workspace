# 🔘 Guía de Botones de Discord Rich Presence

## 🚨 ¿Por qué NO veo mis botones?

### Es un BUG CONOCIDO de Discord

**Discord tiene un bug confirmado**: **NO PUEDES VER TUS PROPIOS BOTONES** en tu Rich Presence. Este no es un problema de tu aplicación, es un problema de Discord que afecta a todos los usuarios.

### ✅ Los botones SÍ funcionan

Aunque **tú no puedas verlos**, los botones **SÍ aparecen para otros usuarios** cuando ven tu perfil de Discord.

## 🔍 Cómo verificar que funcionan

### Método 1: Amigos
1. Pide a un amigo que abra Discord
2. Que vaya a tu perfil o te vea en un servidor
3. Debería ver tus botones debajo de tu Rich Presence

### Método 2: Segunda cuenta
1. Crea otra cuenta de Discord (o usa una existente)
2. Únete al mismo servidor o envía solicitud de amistad
3. Revisa tu perfil desde esa segunda cuenta

### Método 3: Discord Web
1. Abre Discord en un navegador web
2. Inicia sesión con otra cuenta
3. Busca tu perfil principal

## 📱 Dónde aparecen los botones

Los botones aparecen en:
- ✅ **Perfil de usuario** (cuando alguien hace click en tu nombre)
- ✅ **Lista de miembros** del servidor
- ✅ **Mensajes directos** (cuando ven tu actividad)
- ✅ **Lista de amigos** (si tienes actividad visible)

## 🛠️ Tu configuración actual

Tu `config.json` está configurado correctamente:

```json
{
  "discord": {
    "app_id": "1400464001133056111",
    "activities": [
      {
        "name": "commission",
        "details": "💻 Full Stack Developer",
        "state": "Rust • Node.js • Next.js • Vite",
        "large_image": "wow",
        "large_text": "Desarrollo Web & Backend",
        "small_image": "talk_1",
        "small_text": "Disponible",
        "duration_seconds": 3,
        "buttons": [
          {
            "label": "💼 Portfolio",
            "url": "https://tuportfolio.com"
          },
          {
            "label": "🐱 GitHub", 
            "url": "https://github.com/tuusuario"
          }
        ]
      }
    ]
  }
}
```

## 📋 Checklist de verificación

### ✅ Configuración de Discord
- [ ] **Actividad habilitada**: Ve a Configuración → Privacidad → "Compartir estado de actividad"
- [ ] **Discord desktop**: Debe ser la aplicación de escritorio, no navegador
- [ ] **Una sola instancia**: Solo una ventana de Discord abierta
- [ ] **Sin modo invisible**: No estar en modo invisible

### ✅ Aplicación Discord
- [ ] **ID correcto**: `1400464001133056111` debe ser tu Application ID real
- [ ] **Imágenes subidas**: Las imágenes (`wow`, `talk_1`, etc.) deben estar en Discord Developer Portal
- [ ] **URLs válidas**: Los enlaces de botones deben ser URLs completas y válidas

### ✅ Tu aplicación
- [ ] **Conexión exitosa**: Los logs muestran "Discord RPC conectado exitosamente"
- [ ] **Botones detectados**: Los logs muestran "🔘 Configurando X botones"
- [ ] **Sin errores**: No hay errores de conexión en los logs

## 🔧 Troubleshooting

### Si otros tampoco ven los botones:

1. **Verifica URLs**:
   ```
   ❌ Malo: github.com/usuario
   ✅ Bueno: https://github.com/usuario
   ```

2. **Verifica imágenes**:
   - Ve a [Discord Developer Portal](https://discord.com/developers/applications/)
   - Tu aplicación → Rich Presence → Art Assets
   - Asegúrate de que `wow` y `talk_1` están subidas

3. **Verifica Application ID**:
   - Debe ser exactamente: `1400464001133056111`
   - Sin espacios ni caracteres extra

### Si sigues con problemas:

1. **Restart Discord** completamente
2. **Restart tu aplicación**
3. **Espera 1-2 minutos** (Discord puede ser lento actualizando)

## 📚 Referencias oficiales

- [Discord Developer Docs - Rich Presence](https://discord.com/developers/docs/rich-presence/how-to)
- [CustomRP Documentation](https://docs.customrp.xyz/) - También menciona este bug
- [Discord Rich Presence Limits](https://discord.com/developers/docs/rich-presence/best-practices)

## 💡 Tip final

**¡No te preocupes!** Si ves en los logs de tu aplicación que los botones se están configurando correctamente, es porque están funcionando. El que no puedas verlos es 100% normal y esperado debido al bug de Discord.

---

**¿Dudas?** Revisa los logs de tu aplicación - ahora muestran información detallada sobre los botones cuando se configuran.