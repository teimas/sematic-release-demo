# Semantic Release Tool

Este repositorio proporciona **dos implementaciones** de una herramienta de semantic release con integración de **Monday.com**, **JIRA** y generación de notas de versión con IA:

1. **🟨 Versión Node.js** - Script original con interfaz de línea de comandos
2. **🦀 Versión Rust TUI** - Interfaz de usuario de terminal interactiva y moderna

Ambas versiones están configuradas con semantic-release y commitizen para mensajes de commit estandarizados, versionado automático y generación de notas de versión enriquecidas con IA.

## ✨ Nuevas Funcionalidades

- **🎯 Integración JIRA** - Soporte completo para JIRA además de Monday.com
- **🔧 Debug Logging Centralizado** - Todos los errores se registran en `debug.log`
- **📋 Configuración Simplificada** - Archivo `.env.example` con ejemplos completos
- **🔍 UI Limpia** - Sin mensajes de error en pantalla, solo en logs
- **⚙️ Sistema de Tareas Flexible** - Soporta Monday.com, JIRA o ninguno

## Tabla de Contenidos

- [Formato de Mensaje de Commit](#formato-de-mensaje-de-commit)
  - [Plantilla Git de Commit](#-plantilla-git-de-commit)
- [Node.js Version](#-nodejs-version)
- [Rust TUI Version](#-rust-tui-version)
- [Configuración](#️-configuración)
- [Integración con APIs](#-integración-con-apis)
- [Scripts Disponibles](#-scripts-disponibles)

## Formato de Mensaje de Commit

Ambas versiones siguen el mismo formato de mensaje de commit estándar:

```
type(scope): título descriptivo

Descripción detallada de los cambios si es necesario

BREAKING CHANGE: Detalles de cambios que rompen compatibilidad (si aplica)

Test Details: 
- Descripción de prueba 1
- Descripción de prueba 2

Security: Análisis de seguridad o NA

RELATED TASKS:
- [Monday] Título de tarea (ID: 123456789) - Estado
- [JIRA] SMP-123: Título de issue JIRA - Estado
```

**Ejemplo:**
```
feat(8816791718): VERIFACTU - Creación de registros de facturación

Implementación completa de la interfaz para la creación automática 
de registros de facturación en el sistema VERIFACTU

BREAKING CHANGE: El endpoint `/api/facturas` ahora requiere el parámetro `verifactu_enabled`

Test Details:
- Crear registros de facturación para facturas finalizadas
- Verificar que cambios en importes marcan la huella en rojo

Security: Validación de tokens VERIFACTU implementada

RELATED TASKS:
- [Monday] [PE.25.002] VERIFACTU - Registros de facturación (ID: 8816791718) - Done
```

### 📝 Plantilla Git de Commit

Para garantizar consistencia en los mensajes de commit, se incluye una **plantilla de git** que aplica el mismo formato en todos los entornos.

#### Configuración Automática

**Opción 1: Script bash**
```bash
./scripts/setup-commit-template.sh
```

**Opción 2: Comando TUI**
```bash
cargo run -- setup-template
```

**Opciones disponibles en ambos métodos:**
- **Global**: Aplica a todos los repositorios del sistema
- **Local**: Solo para el repositorio actual  
- **Ambos**: Configuración completa

#### Configuración Manual

```bash
# Configurar globalmente
git config --global commit.template ~/.gitmessage

# Configurar solo para el repositorio actual
git config commit.template ~/.gitmessage
```

#### Uso de la Plantilla

**Con plantilla activada:**
```bash
git commit  # Abre editor con plantilla pre-rellenada
```

**Sin plantilla (commits rápidos):**
```bash
git commit -m "mensaje rápido"  # Omite la plantilla
```

#### Beneficios

- ✅ **Consistencia total** entre TUI, CLI y git directo
- ✅ **Documentación integrada** en cada commit
- ✅ **Campos estructurados** garantizados
- ✅ **Adopción gradual** del equipo
- ✅ **Compatible** con cualquier cliente Git

#### Desactivar Plantilla

```bash
# Desactivar globalmente
git config --global --unset commit.template

# Desactivar localmente
git config --unset commit.template
```

---

## 🟨 Node.js Version

La versión original basada en Node.js proporciona scripts de línea de comandos para integración con flujos de trabajo existentes.

### Instalación

```bash
npm install
```

### Uso - Node.js

#### Realizar Commits

```bash
npm run commit        # Versión integrada con Monday.com
npm run commit:simple # Versión básica sin integración
```

**Flujo de trabajo recomendado:**
1. Hacer cambios en el código
2. Ejecutar `npm run commit`
3. Buscar y seleccionar tareas de Monday relacionadas
4. Completar información del commit
5. Revisar y confirmar

#### Publicación de Versiones

```bash
npm run semantic-release
```

Esto realizará:
- Análisis de commits
- Determinación automática de versión
- Generación de changelog
- Creación de etiquetas Git
- Actualización de package.json

#### Generación de Notas de Versión con IA

```bash
npm run release-notes
```

**Proceso automatizado:**
1. Ejecuta semantic-release en modo dry-run
2. Obtiene commits desde la última versión
3. Analiza commits para extraer metadatos
4. Consulta Monday.com para detalles de tareas
5. Genera documento estructurado
6. Procesa con Google Gemini
7. Guarda archivos finales

**Archivos generados:**
- `release-notes-YYYY-MM-DD.md` - Documento estructurado
- `release-notes-YYYY-MM-DD_GEMINI.md` - Versión procesada por IA

#### Búsqueda de Tareas

```bash
npm run monday-selector
```

---

## 🦀 Rust TUI Version

La versión Rust proporciona una **interfaz de usuario de terminal interactiva** moderna y eficiente con características avanzadas.

### 🔧 Compilación y Build

#### Prerrequisitos

1. **Instalar Rust** (si no está instalado):
   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   source ~/.cargo/env
   ```

2. **Verificar instalación**:
   ```bash
   rustc --version
   cargo --version
   ```

#### Compilación para Desarrollo

```bash
# Compilación debug (más rápida, con símbolos de debug)
cargo build

# Ejecutar directamente desde código fuente
cargo run
```

#### Compilación para Producción

```bash
# Compilación optimizada para distribución
cargo build --release

# El binario estará en: target/release/semantic-release-tui
./target/release/semantic-release-tui
```

#### Compilación Cross-Platform

```bash
# Para diferentes arquitecturas (ejemplos)
rustup target add x86_64-pc-windows-gnu
rustup target add aarch64-apple-darwin
rustup target add x86_64-unknown-linux-gnu

cargo build --release --target x86_64-pc-windows-gnu
cargo build --release --target aarch64-apple-darwin
cargo build --release --target x86_64-unknown-linux-gnu
```

#### Optimización de Tamaño

Para un binario más pequeño, modifica `Cargo.toml`:

```toml
[profile.release]
opt-level = 'z'     # Optimizar para tamaño
lto = true          # Link Time Optimization
codegen-units = 1   # Mejor optimización
panic = 'abort'     # Reducir tamaño del binario
strip = true        # Eliminar símbolos de debug
```

Luego compila:
```bash
cargo build --release
```

### Instalación Global

```bash
# Instalar desde el directorio del proyecto
cargo install --path .

# Ahora puedes usar desde cualquier lugar
semantic-release-tui
```

### Uso - Rust TUI

#### Modo Interactivo TUI

```bash
# Ejecutar desde código fuente
cargo run

# O con binario compilado
./target/release/semantic-release-tui
```

**Navegación TUI:**
- `Tab`/`Shift+Tab`: Navegar entre campos
- `↑`/`↓`: Navegar tipos de commit y tareas
- `Enter`: Confirmar selección
- `q`: Salir de la aplicación
- `Esc`: Volver/cancelar

**Teclas especiales en pantalla de commit:**
- `t`: **Análisis Comprensivo IA** - Una llamada API que retorna análisis completo en JSON
- `s`: Buscar tareas de Monday.com
- `c`: Previsualizar mensaje de commit
- `m`: Modo gestión de tareas
- `Space`/`Delete`: Eliminar tareas seleccionadas

#### Comandos de Línea de Comandos

```bash
# Crear commit (abre TUI directamente en pantalla de commit)
cargo run -- commit

# Auto-commit con análisis IA automático
cargo run -- --autocommit

# Generar notas de versión
cargo run -- release-notes

# Buscar tareas de Monday.com
cargo run -- search "nombre de tarea"
```

#### Comandos de Debug

```bash
# Probar conexión Monday.com
cargo run -- debug monday

# Probar conexión JIRA
cargo run -- debug jira

# Probar conexión Gemini AI
cargo run -- debug gemini

# Probar repositorio Git
cargo run -- debug git

# Probar creación de commits con logs detallados
cargo run -- debug commit
```

### 🎯 Características Avanzadas - TUI

#### Análisis IA Comprensivo
- **Una sola llamada API** con prompt detallado
- **Respuesta JSON estructurada**:
  ```json
  {
    "title": "título conciso en español (≤50 chars)",
    "commitType": "tipo de semantic release",
    "description": "descripción técnica exhaustiva (≥150 palabras)",
    "scope": "área del código afectada",
    "securityAnalysis": "vulnerabilidades de seguridad o vacío",
    "breakingChanges": "cambios que rompen compatibilidad o vacío"
  }
  ```
- **Auto-población** de todos los campos simultáneamente
- **Manejo de errores** con fallbacks inteligentes

#### Gestión de Tareas Avanzada
- **Búsqueda en tiempo real** mientras escribes
- **Interfaz multi-selección** con checkboxes visuales
- **Detalles de tareas** (estado, tablero, metadatos)
- **Modo gestión** dedicado con tecla `m`
- **Navegación intuitiva** con indicadores visuales

#### Procesamiento en Background
- **Operaciones no bloqueantes** con threads
- **Actualizaciones de progreso en tiempo real**
- **Mensajes de estado informativos**
- **Manejo seguro de estados** con `Arc<Mutex>`

#### Arquitectura Técnica

**Tecnologías Core:**
- **Ratatui**: Framework de UI terminal con componentes personalizados
- **Tokio**: Runtime async para llamadas concurrentes de API
- **Git2**: Operaciones y análisis de repositorio Git
- **Reqwest**: Cliente HTTP para comunicaciones API
- **Serde**: Serialización/deserialización JSON
- **Crossterm**: Control de terminal cross-platform
- **Clap**: Parsing de argumentos de línea de comandos

**Componentes Clave:**
- **Operaciones Background**: Gestión de estado thread-safe
- **Gestión de Estado UI**: Seguimiento comprensivo de interacciones
- **Manejo de Errores**: Fallbacks elegantes y mensajes amigables
- **Integración Git**: Análisis avanzado de commits y operaciones

---

## ⚙️ Configuración

Ambas versiones comparten la misma configuración almacenada en `.env`.

### 🚀 Configuración Rápida

1. **Copia el archivo de ejemplo:**
   ```bash
   cp .env.example .env
   ```

2. **Edita `.env` con tus valores reales**

3. **Configura automáticamente:**
   ```bash
   # Node.js
   npm run config
   
   # Rust TUI
   cargo run -- config
   ```

### 📋 Variables de Entorno

#### Obligatorias
- `GEMINI_TOKEN` - Token de API de Google Gemini (requerido para IA)

#### Sistema de Tareas (Opcional - elige uno)
**Monday.com:**
- `MONDAY_API_TOKEN` - Token de API de Monday.com
- `MONDAY_BOARD_ID` - ID del tablero principal (opcional)

**JIRA:**
- `JIRA_URL` - URL de tu instancia JIRA (sin slash final)
- `JIRA_USERNAME` - Tu nombre de usuario JIRA (email)
- `JIRA_API_TOKEN` - Token de API de JIRA
- `JIRA_PROJECT_KEY` - Clave del proyecto JIRA (ej: SMP, PROJ)

#### Configuración Avanzada (Opcional)
- `DEBUG` - Habilitar logging debug (true/false)
- `LOG_LEVEL` - Nivel de logging (error, warn, info, debug, trace)
- `RELEASE_NOTES_TEMPLATE` - Ruta a plantilla personalizada

### 🎯 Escenarios de Configuración

#### Escenario 1: Solo Gemini AI (mínimo)
```bash
GEMINI_TOKEN=tu_token_gemini_aqui
```

#### Escenario 2: Gemini + Monday.com
```bash
GEMINI_TOKEN=tu_token_gemini_aqui
MONDAY_API_TOKEN=tu_token_monday_aqui
MONDAY_BOARD_ID=1234567890
```

#### Escenario 3: Gemini + JIRA
```bash
GEMINI_TOKEN=tu_token_gemini_aqui
JIRA_URL=https://tuempresa.atlassian.net
JIRA_USERNAME=tu.email@empresa.com
JIRA_API_TOKEN=tu_token_jira_aqui
JIRA_PROJECT_KEY=TU_PROYECTO
```

#### Escenario 4: Configuración completa
```bash
# Nota: Si ambos están configurados, JIRA tiene prioridad
GEMINI_TOKEN=tu_token_gemini_aqui
MONDAY_API_TOKEN=tu_token_monday_aqui
MONDAY_BOARD_ID=1234567890
JIRA_URL=https://tuempresa.atlassian.net
JIRA_USERNAME=tu.email@empresa.com
JIRA_API_TOKEN=tu_token_jira_aqui
JIRA_PROJECT_KEY=TU_PROYECTO
```

### 🔑 Obtener Claves API

#### Google Gemini API Key (Obligatorio)
1. Ve a https://makersuite.google.com/app/apikey
2. Crea un nuevo proyecto o selecciona uno existente
3. Genera una API key
4. Copia la clave generada

#### Monday.com API Key (Opcional)
1. Ve a https://youraccount.monday.com/admin/integrations/api
2. Genera un nuevo token
3. Copia el token generado

#### JIRA API Token (Opcional)
1. Ve a https://id.atlassian.com/manage-profile/security/api-tokens
2. Crea un nuevo token de API
3. Copia el token generado
4. Usa tu email como username

### 🐛 Debug y Troubleshooting

#### Sistema de Logging Centralizado
Todos los errores y mensajes de debug se escriben en `debug.log`:

```bash
# Ver logs en tiempo real
tail -f debug.log

# Ver logs específicos de un componente
grep "\[JIRA\]" debug.log
grep "\[GEMINI\]" debug.log
grep "\[RELEASE-NOTES\]" debug.log
```

#### Problemas Comunes

**JIRA:**
- ✅ URL sin slash final: `https://empresa.atlassian.net`
- ✅ Project key en mayúsculas: `SMP`, `PROJ`
- ✅ Username debe ser tu email
- ✅ API token válido y con permisos

**Monday.com:**
- ✅ Board ID debe ser solo números
- ✅ API token con permisos de lectura
- ✅ Cuenta debe tener acceso al board

**Gemini:**
- ✅ API key válida y activa
- ✅ Cuotas de API disponibles
- ✅ Conexión a internet estable

---

## 🔗 Integración con APIs

### Gestión de Tareas (Opcional)
La herramienta soporta **múltiples sistemas de gestión de tareas**:

#### Monday.com API
- **Búsqueda global**: Busca en todos los tableros accesibles
- **Búsqueda específica**: Busca en tablero específico si está configurado
- **Detalles de tareas**: Información completa incluyendo:
  - Título, estado y metadatos
  - Información de tableros y URLs
  - Actualizaciones y actividad
  - Valores de columnas personalizadas

#### JIRA API
- **Búsqueda con JQL**: Consultas avanzadas con JIRA Query Language
- **Filtrado por proyecto**: Búsqueda automática en el proyecto configurado
- **Información completa de issues**:
  - Título, descripción y estado
  - Tipo de issue, prioridad y asignado
  - Componentes, etiquetas y fechas
  - Reporter y proyecto asociado
- **Soporte para issue keys**: SMP-123, PROJ-456, etc.

#### Sistema Flexible
- **Sin sistema**: Funciona perfectamente sin configurar tareas
- **Prioridad JIRA**: Si ambos están configurados, JIRA tiene prioridad
- **Detección automática**: La herramienta detecta qué sistema usar
- **Fallbacks elegantes**: Si un sistema falla, continúa funcionando

### Google Gemini API
- **Soporte dual de modelos**:
  - Primario: `gemini-2.5-pro-preview-06-05`
  - Fallback: `gemini-2.0-flash`
- **Ingeniería de prompts avanzada** para análisis precisos
- **Parsing robusto de respuestas JSON**
- **Recuperación de errores** con fallbacks automáticos
- **Análisis de seguridad** comprensivo
- **Debug logging**: Errores detallados en `debug.log`

---

## 📋 Scripts Disponibles

### Node.js Scripts

| Comando | Descripción |
|---------|-------------|
| `npm run commit` | Commit con integración Monday.com |
| `npm run commit:simple` | Commit básico sin integración |
| `npm run semantic-release` | Crear nueva versión |
| `npm run config` | Configurar APIs |
| `npm run monday-selector` | Buscar tareas Monday.com |
| `npm run release-notes` | Generar notas con IA |

### Setup Scripts

| Comando | Descripción |
|---------|-------------|
| `./scripts/setup-commit-template.sh` | Configurar plantilla git de commits (script bash) |
| `cargo run -- setup-template` | Configurar plantilla git de commits (comando TUI) |

### Rust TUI Commands

| Comando | Descripción |
|---------|-------------|
| `cargo run` | Modo TUI interactivo |
| `cargo run -- commit` | Commit directo |
| `cargo run -- --autocommit` | Auto-commit con IA |
| `cargo run -- release-notes` | Generar notas de versión |
| `cargo run -- search "query"` | Buscar tareas (Monday.com/JIRA) |
| `cargo run -- setup-template` | Configurar plantilla git de commits |
| `cargo run -- debug [monday\|gemini\|git\|commit\|jira]` | Debug específico |
| `cargo run -- config` | Configurar APIs |

---

## 🆚 Comparación de Versiones

| Característica | Node.js | Rust TUI |
|----------------|---------|----------|
| **Interfaz** | CLI Scripts | TUI Interactiva |
| **Rendimiento** | Bueno | Excelente |
| **Experiencia de Usuario** | Funcional | Rica e intuitiva |
| **Análisis IA** | Básico | Avanzado con JSON |
| **Gestión de Tareas** | Secuencial | Visual con multi-selección |
| **Soporte JIRA** | No | ✅ Completo |
| **Soporte Monday.com** | ✅ Completo | ✅ Completo |
| **Debug Logging** | Básico | ✅ Centralizado |
| **Background Processing** | No | Sí, con progress |
| **Configuración** | Compartida | Compartida |
| **Mantenimiento** | JavaScript | Rust (type-safe) |
| **Distribución** | npm | Binario independiente |

---

## 🗂️ Archivos de Configuración

- `.env.example` - **Plantilla de configuración con ejemplos completos**
- `.env` - Variables de entorno (APIs) - **NO incluir en git**
- `debug.log` - **Logs centralizados de errores y debug**
- `.cz-config.js` - Configuración de Commitizen
- `.releaserc.json` - Configuración de Semantic-release  
- `Cargo.toml` - Configuración del proyecto Rust
- `package.json` - Configuración del proyecto Node.js

---

## 🚀 Recomendaciones de Uso

### Para Desarrollo Diario
**Rust TUI** - Interfaz más rica y experiencia superior

### Para CI/CD y Automatización  
**Node.js** - Mejor integración con pipelines existentes

### Para Equipos Nuevos
**Rust TUI** - Curva de aprendizaje más suave

### Para Integración Legacy
**Node.js** - Compatibilidad con scripts existentes 

  ████████ ███████ ██ ███    ███  █████  ███████
     ██    ██      ██ ████  ████ ██   ██ ██     
     ██    █████   ██ ██ ████ ██ ███████ ███████
     ██    ██      ██ ██  ██  ██ ██   ██      ██
     ██    ███████ ██ ██      ██ ██   ██ ███████ 