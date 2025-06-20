# [5.0.0](https://github.com/teimas/sematic-release-demo/compare/v4.5.0...v5.0.0) (2025-06-20)


### Bug Fixes

* correct git changes detection logic in comprehensive analysis ([5b9f32a](https://github.com/teimas/sematic-release-demo/commit/5b9f32aa405ca37a3f37b5cc9628445f19b59e22))
* ensure release notes only include commits since last tag ([2ea432f](https://github.com/teimas/sematic-release-demo/commit/2ea432fabd1fa99b31cd40e1a825c3a7c6c93b16))


### Code Refactoring

* **background_operations, observability, error, services, git, types:** Refactoriza y simplifica la base del código ([3c08791](https://github.com/teimas/sematic-release-demo/commit/3c087919608e7fbcd49a8a0c3a612844998c13f4))
* **core, deps, types:** Refactoriza la base para una nueva gestión de estado ([d143636](https://github.com/teimas/sematic-release-demo/commit/d143636df18d05ffa85eae9352b3c0f66e29155f))
* **observability, error-handling, core, services:** implementa errores estructurados y observabilidad ([5e3e0a7](https://github.com/teimas/sematic-release-demo/commit/5e3e0a73def4355c70af5d56c253af26125ff490))


### Features

* **async, app, ui, state:** Añade gestor de tareas asíncrono para UI reactiva ([6e8bd76](https://github.com/teimas/sematic-release-demo/commit/6e8bd7680cb772103316404b4c784137c9e9df1b))


### BREAKING CHANGES

* **background_operations, observability, error, services, git, types:** Este commit introduce numerosos cambios que rompen la compatibilidad interna del código, aunque la interfaz de línea de comandos (CLI) principal permanezca funcional. El impacto se concentra en las APIs internas del crate.

1.  **Módulo `background_operations`**: Se han eliminado las funciones públicas `get_status`, `cancel_operation`, `cancel_all_operations`, `get_active_operations`, `broadcast_progress`, `broadcast_completion`, y `broadcast_error` del `BackgroundTaskManager`. Cualquier componente que dependiera de ellas para la cancelación o el seguimiento detallado de tareas fallará en la compilación.
2.  **Enum `BackgroundEvent`**: Se han eliminado las variantes `SemanticReleaseProgress`, `SemanticReleaseCompleted`, `SemanticReleaseError` y `OperationCancelled`. El código que maneje estos eventos, como en `app.rs`, ya no compilará.
3.  **Enum `OperationStatus`**: Se ha simplificado, eliminando los datos asociados a cada variante (`progress`, `result`, `error`). El código que haga pattern matching esperando estos datos se romperá.
4.  **Módulo `error`**: Se han eliminado múltiples variantes del enum `SemanticReleaseError` y sus constructores asociados (ej: `ServiceError`, `UiError`, `BackgroundOperationError`). Todo el código que manejaba o creaba estos tipos de error específicos deberá ser actualizado.
5.  **Estructura `GitCommit`**: Se han eliminado los campos `author_name`, `author_email`, `commit_date`, `test_details`, `security`, `monday_task_mentions` y `jira_task_mentions`. Cualquier parte de la aplicación que acceda a estos campos, especialmente en la UI, se romperá.
6.  **Módulos de Servicios (`jira.rs`, `monday.rs`)**: Se han eliminado las funciones públicas `get_task_details`. El código que intentaba enriquecer la información de las tareas a partir de sus IDs ya no funcionará.
7.  **Módulo `observability`**: Las funciones públicas de inicialización `init_observability` e `init_development_observability` han sido eliminadas. El punto de entrada de la aplicación (`main.rs`) ha sido modificado para reflejar esto.

Test Details: Se deben realizar pruebas manuales exhaustivas para asegurar que la aplicación sigue siendo estable y funcional después de esta refactorización masiva y la eliminación de funcionalidades.

1.  **Pruebas de Arranque y Estabilidad General**:
    *   Ejecuta la aplicación con `cargo run`.
    *   Verifica que la TUI se inicie correctamente sin pánicos ni errores en la consola, especialmente debido a la eliminación del sistema de observabilidad.
    *   Navega por todas las vistas y paneles disponibles (lista de commits, detalles, etc.) y confirma que no hay crashes.

2.  **Verificación de Operaciones en Segundo Plano**:
    *   Inicia una operación que consuma tiempo, como "Análisis Exhaustivo".
    *   Observa el área de mensajes de estado en la UI. Deberías ver un indicador genérico de "Cargando" o similar, pero NO mensajes de progreso detallados (ej: "🚀 Analizando commit X..."), ya que esta funcionalidad fue eliminada.
    *   Confirma que la operación finaliza y el estado vuelve a la normalidad con un mensaje de éxito o error simple.

3.  **Verificación del Panel de Detalles del Commit**:
    *   Selecciona varios commits en la lista principal.
    *   En el panel de detalles, verifica que los campos eliminados (`Author`, `Commit Date`, `Test Details`, `Security Notes`) ya NO se muestran. La UI debe haberse adaptado para mostrar solo la información disponible en la nueva estructura `GitCommit` (hash, tipo, scope, descripción, cuerpo).

4.  **Pruebas de la Interfaz de Línea de Comandos (CLI)**:
    *   Ejecuta los subcomandos que no usan la TUI, por ejemplo `cargo run -- analyze version`.
    *   Provoca un error (ej: ejecutándolo fuera de un repositorio git) y verifica que el mensaje de error se imprime de forma clara en la consola, confirmando que el cambio de `log_error_to_console` a `log_user_message` funciona como se espera.

5.  **Verificación de Archivos de Log**:
    *   Busca la carpeta `logs`.
    *   Abre un archivo de log generado tras ejecutar la aplicación.
    *   Confirma que el formato ya no es JSON ni tiene una estructura de árbol. Debería ser un formato de texto plano y más simple.
    *   Confirma que durante la ejecución normal, la consola permanece limpia de logs de `INFO` o `DEBUG`.

Security: N/A

Migraciones Lentas: N/A

Partes a Ejecutar: N/A

JIRA TASKS: N/A
* **core, deps, types:** Se han eliminado las funciones públicas `log_warning` y `log_info` del módulo `utils`. Cualquier parte del código que dependiera de estas funciones de utilidad para el logging deberá ser actualizada para usar otras alternativas como `log_debug` o `tracing::warn`. Adicionalmente, se han modificado los re-exports en el archivo principal `src/lib.rs`, lo que podría afectar a cómo otros módulos internos acceden a ciertas funcionalidades, requiriendo la actualización de las rutas de importación.

Test Details: Se requiere una serie de pruebas manuales exhaustivas para validar esta refactorización fundamental:

1.  **Prueba de Compilación y Arranque:**
    *   Paso 1: Realizar una compilación limpia del proyecto (`cargo clean && cargo build`).
    *   Paso 2: Ejecutar la aplicación.
    *   Resultado esperado: La aplicación debe compilar sin errores y arrancar correctamente, demostrando que las nuevas dependencias y los cambios en la estructura no han roto el build básico.

2.  **Regresión Funcional Completa:**
    *   Paso 1: Realizar el flujo completo de generación de notas de lanzamiento, incluyendo la conexión a servicios, la obtención de commits y la generación del archivo final.
    *   Paso 2: Interactuar con todas las partes de la UI (paneles, menús, entradas de texto).
    *   Resultado esperado: Todas las funcionalidades existentes deben operar exactamente como antes. No debe haber panics, errores inesperados o cambios de comportamiento. Esto verifica que los cambios subyacentes no han introducido regresiones.

3.  **Verificación de Operaciones en Segundo Plano:**
    *   Paso 1: Iniciar una operación que se ejecute en segundo plano, como la generación de notas de lanzamiento con análisis de IA activado.
    *   Paso 2: Observar el estado de la operación en la interfaz de usuario.
    *   Resultado esperado: La tarea debe iniciarse, mostrar su progreso y completarse (o fallar) correctamente, actualizando la UI como corresponde. Esto valida que la comunicación entre hilos no se ha visto afectada.

4.  **Verificación de Archivos de Log:**
    *   Paso 1: Realizar varias acciones en la aplicación que generen logs (errores, éxitos, etc.).
    *   Paso 2: Revisar el archivo `debug.log`.
    *   Resultado esperado: El archivo de log debe seguir registrando eventos correctamente. No deben aparecer llamadas a las funciones eliminadas `log_warning` o `log_info`, confirmando que su eliminación fue completa.

Security: N/A

Migraciones Lentas: N/A

Partes a Ejecutar: N/A

JIRA TASKS: N/A
* **async, app, ui, state:** Se ha modificado fundamentalmente la forma en que se inician y gestionan las operaciones en segundo plano. La API interna para iniciar tareas como la generación de notas de versión ha cambiado por completo. El antiguo método de crear un `ReleaseNotesAnalysisState` con `Arc<Mutex<T>>` y pasarlo a un nuevo hilo `std::thread` ha sido eliminado. Ahora, todo el código interno debe usar la nueva interfaz `BackgroundTaskManager::start_..._generation()`. Aunque se mantiene la compatibilidad hacia atrás en la estructura de estado `App` para una migración gradual, cualquier código que dependiera del mecanismo de polling anterior dejará de funcionar como se esperaba. La función síncrona `generate_release_notes_with_ai_analysis` ahora tiene una implementación básica y se considera legada, lo que podría afectar a componentes internos que dependieran de su comportamiento completo.

Test Details: 1. **Generación de Notas de Versión (Caso de Éxito)**:
   - Inicia la acción para generar notas de versión.
   - Verifica que la UI muestre inmediatamente un mensaje de carga como '🚀 Iniciando...'.
   - Observa si aparecen mensajes de progreso en tiempo real (ej: '📋 Obteniendo commits...', '🤖 Analizando con IA...').
   - Al finalizar, confirma que se muestra un mensaje de éxito como '✅ Notas de versión generadas' y la aplicación vuelve a su estado normal.

2. **Cancelación de Operación**:
   - Inicia una operación larga (generación de notas de versión).
   - Mientras la operación está en progreso, presiona la tecla o comando de cancelación (generalmente 'q' o 'Esc').
   - Verifica que la operación se detenga y la UI muestre un mensaje '❌ Operación cancelada'.

3. **Manejo de Errores Asíncronos**:
   - Configura la aplicación con una clave de API de Gemini/IA inválida para forzar un error.
   - Inicia la generación de notas de versión.
   - Confirma que, tras el intento de llamada a la API, la UI muestra un mensaje de error específico y claro, como '❌ Error en generación: Fallo en la autenticación con el servicio de IA'.

4. **Integración de Análisis de IA en Formulario**:
   - Ejecuta la operación 'Análisis Completo'.
   - Tras recibir el mensaje '✅ Análisis completado', navega a la pantalla del formulario de commit.
   - Verifica que los campos (tipo de commit, título, descripción, ámbito, etc.) se han rellenado automáticamente con los datos proporcionados por la IA.

5. **Verificación del Bugfix de JIRA Key**:
   - Crea manualmente un commit cuyo mensaje tenga un ámbito malformado, por ejemplo: `feat(PROJ-123,unrelated-text): new feature`.
   - Ejecuta la generación de notas de versión.
   - Revisa las notas generadas o los logs para confirmar que solo `PROJ-123` fue identificado como una tarea de JIRA y que `unrelated-text` fue ignorado.

6. **Actualización del Roadmap**:
   - Revisa el documento `ROADMAP.md` dentro del proyecto.
   - Confirma que la sección 'Phase 1.2: Async Runtime Modernization' está marcada como 'COMPLETED' y que las tareas pendientes asociadas han sido eliminadas o marcadas como completadas.

Security: N/A

Migraciones Lentas: N/A

Partes a Ejecutar: N/A

JIRA TASKS: N/A
* **observability, error-handling, core, services:** Se ha modificado la firma de la mayoría de las funciones públicas y privadas que pueden devolver un error. El tipo de retorno ha cambiado de `anyhow::Result<T>` a un nuevo tipo `crate::error::Result<T>`, que es un alias para `Result<T, SemanticReleaseError>`. Cualquier código externo que consuma esta aplicación como una biblioteca y dependa de los tipos de error anteriores se romperá y deberá ser actualizado para manejar el nuevo enum de error `SemanticReleaseError`.

Test Details: Se deben realizar las siguientes pruebas manuales para verificar la integridad de los cambios:

1.  **Pruebas de Manejo de Errores (Rutas de Fallo):**
    *   Ejecuta `terco commit` en un directorio que no sea un repositorio Git. Verifica que se muestre un error claro y formateado por `miette` indicando la ausencia del repositorio.
    *   Modifica el `.env` con un `GEMINI_TOKEN` inválido. Ejecuta `terco --autocommit`. Confirma que la aplicación falla con un mensaje de error específico sobre la conexión con la API de Gemini.
    *   Configura una `JIRA_API_TOKEN` incorrecta y ejecuta `terco search "test"`. Valida que el error de API de JIRA se informa correctamente al usuario.
    *   Intenta ejecutar `terco config` y guardar la configuración en un directorio donde no tengas permisos de escritura. Verifica que se muestra un error de I/O claro.

2.  **Pruebas del Sistema de Logging y Observabilidad:**
    *   Ejecuta la aplicación con el nuevo flag `--dev` (ej: `terco --dev version-info`). Verifica que la consola muestra un log jerárquico y coloreado de `tracing-tree`.
    *   Ejecuta la aplicación con el flag `--debug` (ej: `terco --debug search "test"`). Busca el archivo `terco-debug.log` y confirma que contiene logs detallados de nivel DEBUG.
    *   Ejecuta un comando sin flags de depuración (ej: `terco version-info`). Comprueba que el archivo de log se crea y contiene logs de nivel INFO, mientras que la consola solo muestra la salida esperada para el usuario.
    *   Verifica que los comandos que imprimen información al usuario (`search`, `version-info`, etc.) siguen mostrando la información correctamente en la consola, independientemente de la configuración de logging.

3.  **Pruebas de Regresión (Rutas Felices):**
    *   Realiza un flujo completo de `terco config` para configurar todas las credenciales.
    *   Realiza un flujo de `terco commit` y `terco --autocommit` con cambios válidos en el repositorio. Asegúrate de que el commit se crea correctamente.
    *   Ejecuta `terco release-notes` y completa el proceso. Verifica que las notas de versión se generan como antes.
    *   Confirma que todos los flujos principales de la aplicación funcionan sin cambios aparentes en el comportamiento desde la perspectiva del usuario final.

Security: N/A

Migraciones Lentas: N/A

Partes a Ejecutar: N/A

JIRA TASKS: N/A

# [4.5.0](https://github.com/teimas/sematic-release-demo/compare/v4.4.0...v4.5.0) (2025-06-19)


### Features

* **services, app, ui, cli:** integra análisis de pruebas generado por IA ([f15d50a](https://github.com/teimas/sematic-release-demo/commit/f15d50afeccac5d0098b90bab0bb448d3658b7c9))

# [4.4.0](https://github.com/teimas/sematic-release-demo/compare/v4.3.0...v4.4.0) (2025-06-19)


### Features

* **setup, git:** Automatiza URL de repositorio en package.json ([473f5ae](https://github.com/teimas/sematic-release-demo/commit/473f5ae6ef1a3e81147e5a3ac712b170e6f011cd))

# [4.3.0](https://github.com/teimas/sematic-release-demo/compare/v4.2.0...v4.3.0) (2025-06-18)


### Features

* **config, ci:** Mejora la configuración inicial de proyectos Node.js ([1d29c96](https://github.com/teimas/sematic-release-demo/commit/1d29c961e3d418ce0304b50d1134f56daa979ad9))

# [4.2.0](https://github.com/teimas/sematic-release-demo/compare/v4.1.0...v4.2.0) (2025-06-18)


### Features

* **semantic-release, ui, config:** Añade configuración automática de GitHub Actions ([0c066fe](https://github.com/teimas/sematic-release-demo/commit/0c066fe3c8ce9dc1309578ca42f7423c20329e15))

# [4.1.0](https://github.com/teimas/sematic-release-demo/compare/v4.0.1...v4.1.0) (2025-06-18)


### Features

* **config:** Añade creación automática de plantilla de release ([ae2d44a](https://github.com/teimas/sematic-release-demo/commit/ae2d44a425ed60239a45c2f2f327772506e59b6a))

## [4.0.1](https://github.com/teimas/sematic-release-demo/compare/v4.0.0...v4.0.1) (2025-06-18)


### Bug Fixes

* **commit:** Omite pie de página 'BREAKING CHANGE' si no aplica ([56254e4](https://github.com/teimas/sematic-release-demo/commit/56254e4e8250d7344f61c4adfe0cda99788490d6))

# [4.0.0](https://github.com/teimas/sematic-release-demo/compare/v3.0.0...v4.0.0) (2025-06-18)


### Bug Fixes

* clean up unused imports ([772763a](https://github.com/teimas/sematic-release-demo/commit/772763a4542d4e855b0bb531b4b53fe6e6b252f3))
* improve error handling in version detection ([8a36b35](https://github.com/teimas/sematic-release-demo/commit/8a36b35a158b07d73136522cf22bc0dd04898e8c))
* resolve async trait warnings with allow attributes ([0b95ba6](https://github.com/teimas/sematic-release-demo/commit/0b95ba6ce191f44f925fdc0ab638817e8804dbc0))


### Code Refactoring

* **app:** Optimiza adición de carácter de nueva línea ([eddc6ab](https://github.com/teimas/sematic-release-demo/commit/eddc6ab343d32962a744be2d6bf5e78664c7d1fd))


### Features

* add enhanced version detection functionality ([122942f](https://github.com/teimas/sematic-release-demo/commit/122942f03b587493c9a0d3ceef5950e220929138))
* **cli, versioning:** añade comando CLI para mostrar información de versión ([1175470](https://github.com/teimas/sematic-release-demo/commit/1175470a0fbac706219baf3eaa52b4639a1e8ebb))


### Styles

* **main, ui:** Ajusta espaciado en salida y código fuente ([553f5f5](https://github.com/teimas/sematic-release-demo/commit/553f5f50bd0c3e04d0e43c0ae90a9ea487d50e5e))


### BREAKING CHANGES

* **app:** N/A

Test Details: N/A

Security: N/A

Migraciones Lentas: N/A

Partes a Ejecutar: N/A

JIRA TASKS: N/A
* **main, ui:** N/A

Test Details: N/A

Security: N/A

Migraciones Lentas: N/A

Partes a Ejecutar: N/A

JIRA TASKS: N/A
* **cli, versioning:** N/A

Test Details: N/A

Security: N/A

Migraciones Lentas: N/A

Partes a Ejecutar: N/A

JIRA TASKS: N/A

# [3.0.0](https://github.com/teimas/sematic-release-demo/compare/v2.0.0...v3.0.0) (2025-06-18)


### chore

* **ci:** Limpia logs de depuración en workflow de release ([a0064f7](https://github.com/teimas/sematic-release-demo/commit/a0064f71e994715ef3d8ff33268f57f6f035fd1d))


### BREAKING CHANGES

* **ci:** N/A

Test Details: N/A

Security: El código anterior presentaba un riesgo de 'Exposición de Información Confidencial'. Las líneas `echo "GITHUB_TOKEN: $GITHUB_TOKEN"` y `echo $GITHUB_TOKEN` en el archivo `.github/workflows/release.yml` imprimían explícitamente un token de acceso (`GITHUB_TOKEN`) en los logs de ejecución. Aunque GitHub Actions enmascara los secretos, esta práctica es intrínsecamente insegura, ya que el enmascaramiento podría fallar o ser eludido, exponiendo el token. Un token expuesto podría permitir a un actor malicioso realizar acciones no autorizadas en el repositorio, como inyectar código o eliminar ramas. Este commit mitiga directamente este riesgo al eliminar la impresión del token, adhiriéndose a las mejores prácticas de manejo seguro de credenciales.

Migraciones Lentas: N/A

Partes a Ejecutar: N/A

JIRA TASKS: N/A

# [2.0.0](https://github.com/teimas/sematic-release-demo/compare/v1.0.0...v2.0.0) (2025-06-18)


### Bug Fixes

* **release_notes:** Corrige la lógica de extracción de IDs de tarea ([f1618e5](https://github.com/teimas/sematic-release-demo/commit/f1618e5428fcb91775497d5d07de16cc3fef9750))


### chore

* **build, ci, project:** formatea el código y actualiza el workflow de release ([51386da](https://github.com/teimas/sematic-release-demo/commit/51386da17f94c75e73a9c0add57d1895bf26dd9b))
* **build, deps, ci:** Robustece la compilación con OpenSSL dinámico y estático ([3ad4ee8](https://github.com/teimas/sematic-release-demo/commit/3ad4ee809a7397541dd316b20911e2da4f212fee))
* **ci/cd:** Añade log de depuración al workflow de release ([40c94d7](https://github.com/teimas/sematic-release-demo/commit/40c94d7753083ee5bc346443680894006830881a))
* **ci:** Diferencia la compilación de CI para Windows y Unix ([de63c12](https://github.com/teimas/sematic-release-demo/commit/de63c12c4d2eb81c9b11cdffeadd2ae88f9fccb0))


### BREAKING CHANGES

* **ci/cd:** N/A

Test Details: N/A

Security: Se ha detectado una vulnerabilidad de seguridad crítica de tipo 'Exposición de Información Confidencial'. El cambio introduce la línea `echo $GITHUB_TOKEN` en el workflow de `release.yml`. Esta instrucción imprime el valor del secreto `GITHUB_TOKEN` directamente en los logs de ejecución de GitHub Actions. Aunque GitHub Actions intenta enmascarar automáticamente los secretos en los logs, este mecanismo no es infalible y puede ser eludido. La exposición de este token, que generalmente posee permisos de escritura en el repositorio (para crear releases, tags, etc.), permitiría a cualquier persona con acceso a los logs (incluyendo colaboradores con acceso de solo lectura en repositorios públicos) comprometer la integridad del repositorio. Un atacante podría usar el token para publicar versiones maliciosas, eliminar releases existentes o realizar otras acciones no autorizadas en el repositorio, representando un riesgo de seguridad muy elevado. Esta línea debe ser eliminada inmediatamente después de completar la depuración.

Migraciones Lentas: N/A

Partes a Ejecutar: N/A

JIRA TASKS: N/A
* **ci:** N/A

Test Details: N/A

Security: N/A

Migraciones Lentas: N/A

Partes a Ejecutar: N/A

JIRA TASKS: N/A
* **release_notes:** N/A

Test Details: N/A

Security: N/A

Migraciones Lentas: N/A

Partes a Ejecutar: N/A

JIRA TASKS: N/A
* **build, deps, ci:** N/A

Test Details: N/A

Security: El commit modifica la gestión de la dependencia criptográfica OpenSSL. Al priorizar el uso de la librería OpenSSL del sistema, existe un riesgo potencial de que la aplicación se enlace dinámicamente con una versión antigua y vulnerable de OpenSSL si no se actualiza en el entorno de compilación. Sin embargo, este riesgo es mitigado significativamente en el entorno de CI por dos factores: el workflow intenta instalar versiones recientes y, más importante aún, el mecanismo de fallback a una versión "vendored" (compilada desde la fuente, OpenSSL v3.5.0) asegura que se utilice una versión moderna y segura si la del sistema falla o es incompatible. Por lo tanto, aunque se interactúa con una dependencia crítica de seguridad, el cambio está diseñado como una medida de robustez y seguridad para el proceso de build, no introduce una vulnerabilidad directa.

Migraciones Lentas: N/A

Partes a Ejecutar: N/A

JIRA TASKS: N/A
* **build, ci, project:** El workflow de `release.yml` ha sido modificado. El nombre del secreto para la autenticación con GitHub ha cambiado de `GITHUB_TOKEN` a `GH_TOKEN`. El proceso de release fallará si el nuevo secreto `GH_TOKEN` no se configura correctamente en los secretos del repositorio de GitHub. Adicionalmente, el paso de publicación en NPM ha sido deshabilitado al comentar la variable `NPM_TOKEN`, lo que significa que las nuevas versiones ya no se publicarán en el registro de NPM hasta que se revierta este cambio.

Test Details: N/A

Security: N/A

Migraciones Lentas: N/A

Partes a Ejecutar: N/A

JIRA TASKS: N/A
