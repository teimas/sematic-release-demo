# Demo de Semantic Release

Este repositorio está configurado con semantic-release y commitizen para mensajes de commit estandarizados y versionado automático.

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

En lugar de usar `git commit` directamente, utiliza:

```bash
npm run commit
```

Esto iniciará un asistente interactivo que te guiará para crear un mensaje de commit que cumpla con el formato requerido.

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

### Integración con Monday.com

Este repositorio incluye integración con la API de Monday.com para búsqueda y gestión de tareas.

#### Configuración de Monday.com

Para configurar el acceso a Monday.com:

```bash
npm run config
```

Esto te solicitará:
- Tu token de API de Monday.com
- El ID del tablero principal (opcional)

La configuración se guardará en un archivo `.env` que no se incluirá en el control de versiones por seguridad.

#### Búsqueda de Tareas

Para buscar tareas en Monday.com:

```bash
npm run search-task
```

Este script te permitirá buscar tareas por nombre en el tablero configurado o, si no se ha especificado un tablero, en todos los tableros accesibles.

## Archivos de Configuración

- `.cz-config.js`: Configuración de Commitizen
- `.releaserc.json`: Configuración de Semantic-release
- `.env`: Configuración de acceso a Monday.com (creado automáticamente, no versionado) 