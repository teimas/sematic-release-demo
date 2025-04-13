# Demo de Semantic Release

Este repositorio está configurado con semantic-release y commitizen para mensajes de commit estandarizados, versionado automático y generación de notas de versión enriquecidas con IA.

## Formato de Mensaje de Commit

El formato del mensaje de commit sigue esta plantilla:

```
refs mXXXXXXXXXX [PE.XX.XXX] TITLE

Detailed description of the changes if necessary

Test 1: Description of test 1
Test 2: Description of test 2

Security: NA
Change-Id: IaXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX
```

Ejemplo:
```
refs m8816791718 [PE.25.002] VERIFACTU. Creación de registros de facturación (Interfaz)

Descripción libre de lo que hace 
este commit si fuese necesario o interesante

Test 1: Crear registros de facturación para facturas finalizadas.
Test 2: Comprobar que si se modifica algún dato que cambia el importe, en la factura, la huella del registro de facturación se marca en rojo.

Security: NA
Change-Id: Ia32043bb5d86dacf73fa6a96190473501b4a9ccb
```

## Cómo Usar

### Realizar Commits

En lugar de usar `git commit` directamente, utiliza uno de estos comandos:

```bash
npm run commit        # Versión integrada con Monday
npm run commit:simple # Versión básica sin integración con Monday
```

Este comando proporciona la mejor experiencia:
1. Te permite buscar tareas en Monday.com relacionadas con tu commit
2. Puedes seleccionar múltiples tareas usando la barra espaciadora
3. Automáticamente extrae los IDs de las tareas como scope del commit
4. Incluye los detalles completos de las tareas (título, ID y URL) en el mensaje

#### Flujo de Trabajo con Monday.com

El flujo de trabajo recomendado es:

1. Hacer cambios en el código
2. Ejecutar `npm run commit`
3. Buscar y seleccionar las tareas de Monday relacionadas con tus cambios
4. Completar el resto de la información del commit
5. Revisar el mensaje generado y confirmar

El mensaje final incluirá:
- Tipo de cambio (feat, fix, etc.)
- Scope con los IDs de las tareas seleccionadas (ej: `(8851673176|8872179232)`)
- Título descriptivo
- Información detallada del commit
- Sección "MONDAY TASKS" con los detalles y URLs de las tareas

### Publicación de Versiones

Para crear una versión:

```bash
npm run semantic-release
```

Esto hará:
1. Analizar tus commits
2. Determinar la próxima versión basada en versionado semántico
3. Generar un registro de cambios
4. Crear una nueva etiqueta Git
5. Actualizar package.json con la nueva versión

### Generación de Notas de Versión con IA

Este repositorio incluye una potente funcionalidad para generar notas de versión detalladas utilizando Google Gemini:

```bash
npm run release-notes
```

Este comando:
1. Ejecuta semantic-release en modo dry-run para determinar la próxima versión
2. Obtiene todos los commits desde la última versión etiquetada
3. Analiza cada commit para extraer:
   - Tipo de cambio (feat, fix, etc.)
   - Scope y descripción
   - Breaking changes
   - Detalles de pruebas
   - Información de seguridad
   - Referencias a tareas de Monday.com
4. Consulta la API de Monday.com para obtener detalles completos de las tareas mencionadas
5. Genera un documento estructurado con toda esta información
6. Envía este documento a Google Gemini para crear notas de versión profesionales
7. Guarda tanto el documento original como la respuesta de Gemini

#### Configuración de Google Gemini

Para configurar el acceso a Google Gemini:

```bash
npm run config
```

Durante la configuración, además de los datos de Monday.com, se te solicitará:
- Tu token de API de Google Gemini

El token se guardará en el archivo `.env`.

#### Archivos Generados

El proceso genera dos archivos en la carpeta `release-notes/`:

1. `release-notes-YYYY-MM-DD.md`: Documento estructurado con todos los datos extraídos
2. `release-notes-YYYY-MM-DD_GEMINI.md`: Notas de versión generadas por Gemini en español

Las notas generadas por Gemini incluyen:
- Resumen ejecutivo de la versión
- Lista organizada de nuevas funcionalidades
- Lista de correcciones y mejoras
- Cambios que rompen compatibilidad
- Detalles de las tareas de Monday.com relacionadas
- Información completa de los commits

#### Personalización

El formato y contenido de las notas de versión se puede personalizar modificando el script `scripts/prepare-release-notes.js`. Puedes ajustar:
- El idioma de las notas generadas
- La estructura del documento enviado a Gemini
- Los parámetros de generación (temperatura, longitud, etc.)
- El formato de los archivos de salida

### Integración con Monday.com

Este repositorio incluye integración con la API de Monday.com para búsqueda y gestión de tareas.

#### Configuración de Monday.com

Para configurar el acceso a Monday.com:

```bash
npm run config
```

Esto te solicitará:
- Tu token de API de Monday.com
- Tu subdominio de Monday.com (ej: "miempresa" para miempresa.monday.com)
- El ID del tablero principal (opcional)
- Tu token de API de Google Gemini (opcional)

La configuración se guardará en un archivo `.env` que no se incluirá en el control de versiones por seguridad.

#### Búsqueda de Tareas

Para buscar tareas en Monday.com de forma independiente:

```bash
npm run monday-selector
```

Este script te permitirá buscar tareas por nombre en el tablero configurado o, si no se ha especificado un tablero, en todos los tableros accesibles. Además, muestra la URL directa para cada tarea.

#### Commits Vinculados a Tareas de Monday

El script `commit` ofrece una integración completa entre tus commits y las tareas de Monday.com:

```bash
npm run commit
```

**Características:**

- **Búsqueda integrada**: Busca tareas de Monday directamente durante el proceso de commit
- **Selección múltiple**: Selecciona varias tareas relacionadas con tus cambios
- **Auto-extracción de scope**: Los IDs de las tareas se utilizan automáticamente como scope
- **Enlaces completos**: Incluye título, ID y URL directa de cada tarea
- **Preguntas estándar**: Mantiene todas las preguntas del formato de commit requerido (Test Details, Security, etc.)

**Formato de Mensaje Generado:**

```
feat(8851673176|8872179232): Título descriptivo

Descripción detallada del cambio

BREAKING CHANGE: Detalles de cambios que rompen compatibilidad (si aplica)

Test Details: 
- Detalle de prueba 1
- Detalle de prueba 2

Security: NA

Refs: mXXXXXXXXXX

Change-Id: IaXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX

MONDAY TASKS:
- [PE.25.002] Tarea 1 (ID: 8851673176, URL: teimas.monday.com/boards/1013914950/pulses/8851673176)
- [PE.25.002] Tarea 2 (ID: 8872179232, URL: teimas.monday.com/boards/1013914950/pulses/8872179232)
```

Este formato facilita:
- Referencia directa a las tareas de Monday por su ID
- Enlaces clickeables para acceder rápidamente a cada tarea
- Búsqueda posterior de commits por ID de tarea
- Generación de notas de versión enriquecidas

## Archivos de Configuración

- `.cz-config.js`: Configuración de Commitizen
- `.releaserc.json`: Configuración de Semantic-release
- `.env`: Configuración de acceso a Monday.com y Google Gemini (creado automáticamente, no versionado)

## Scripts Disponibles

| Comando | Descripción |
|---------|-------------|
| `npm run commit` | Crea un commit con integración con Monday.com |
| `npm run commit:simple` | Crea un commit básico sin integración con Monday |
| `npm run semantic-release` | Ejecuta semantic-release para crear una nueva versión |
| `npm run config` | Configura la integración con Monday.com y Google Gemini |
| `npm run monday-selector` | Busca tareas en Monday.com |
| `npm run release-notes` | Genera notas de versión detalladas con Google Gemini | 