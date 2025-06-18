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
