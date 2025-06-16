# Semantic Release Tool

Este repositorio proporciona **dos implementaciones** de una herramienta de semantic release con integración de Monday.com y generación de notas de versión con IA:

1. **🟨 Versión Node.js** - Script original con interfaz de línea de comandos
2. **🦀 Versión Rust TUI** - Interfaz de usuario de terminal interactiva y moderna

Ambas versiones están configuradas con semantic-release y commitizen para mensajes de commit estandarizados, versionado automático y generación de notas de versión enriquecidas con IA.

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

MONDAY TASKS:
- Título de tarea (ID: 123456789) - Estado
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

MONDAY TASKS:
- [PE.25.002] VERIFACTU - Registros de facturación (ID: 8816791718) - Done
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

### Configuración Inicial

**Node.js:**
```bash
npm run config
```

**Rust TUI:**
```bash
cargo run -- config
```

**Variables configuradas:**
- `MONDAY_API_KEY` - Token de API de Monday.com
- `MONDAY_ACCOUNT_SLUG` - Subdominio de Monday.com
- `MONDAY_BOARD_ID` - ID del tablero principal (opcional)
- `GEMINI_TOKEN` - Token de API de Google Gemini

### Obtener Claves API

#### Monday.com API Key
1. Ve a https://youraccount.monday.com/admin/integrations/api
2. Genera un nuevo token
3. Copia el token generado

#### Google Gemini API Key
1. Ve a https://makersuite.google.com/app/apikey
2. Crea un nuevo proyecto o selecciona uno existente
3. Genera una API key
4. Copia la clave generada

---

## 🔗 Integración con APIs

### Monday.com API
- **Búsqueda global**: Busca en todos los tableros accesibles
- **Búsqueda específica**: Busca en tablero específico si está configurado
- **Detalles de tareas**: Información completa incluyendo:
  - Título, estado y metadatos
  - Información de tableros y URLs
  - Actualizaciones y actividad
  - Valores de columnas personalizadas

### Google Gemini API
- **Soporte dual de modelos**:
  - Primario: `gemini-2.5-pro-preview-06-05`
  - Fallback: `gemini-2.0-flash`
- **Ingeniería de prompts avanzada** para análisis precisos
- **Parsing robusto de respuestas JSON**
- **Recuperación de errores** con fallbacks automáticos
- **Análisis de seguridad** comprensivo

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
| `cargo run -- search "query"` | Buscar tareas |
| `cargo run -- setup-template` | Configurar plantilla git de commits |
| `cargo run -- debug [monday\|gemini\|git\|commit]` | Debug específico |
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
| **Background Processing** | No | Sí, con progress |
| **Configuración** | Compartida | Compartida |
| **Mantenimiento** | JavaScript | Rust (type-safe) |
| **Distribución** | npm | Binario independiente |

---

## 🗂️ Archivos de Configuración

- `.cz-config.js` - Configuración de Commitizen
- `.releaserc.json` - Configuración de Semantic-release  
- `.env` - Variables de entorno (APIs)
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