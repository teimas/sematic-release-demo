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
