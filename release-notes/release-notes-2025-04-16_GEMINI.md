# Actualización Teixo versión tag-teixo-20250416-1.112.0

# **Información para N1**

Tickets que se solucionan en esta actualización

| Desarrollo | IDTarea | Support Bee |
|---|---|---|
| BUG. Correción de informe "Resumen anual de entradas por centro" | m8817155664 | https://teimas.supportbee.com/tickets/51279217 |
| BUG. Error al eliminar descuentos de líneas | m8874527315 | https://teimas.supportbee.com/tickets/51385685, https://teimas.supportbee.com/tickets/51414391, https://teimas.supportbee.com/tickets/51366346 |
| BUG. En el listado "Listado de líneas de albarán entre fechas de entrega" se muestran albaranes facturados cuando no lo están | m8884168044 | https://teimas.supportbee.com/tickets/51429571 |


# **Información técnica**

### Responsable despliegue

LudoBermejoES

### Etiquetas

tag-teixo-20250416-1.112.0

### Migraciones lentas

| IDTarea | Fichero | Tiempos |
|---|---|---|
|  |  |  |

### Partes a ejecutar

| IDTarea | Enlace a Script |
|---|---|---|
|  |  |


## **Cambios para entorno de producción**

## **Correcciones**

- **BUG. Correción de informe "Resumen anual de entradas por centro" (ID: 8817155664):** Corregido un problema en el informe "Resumen anual de entradas por centro".
- **BUG. Error al eliminar descuentos de líneas (ID: 8874527315):** Solucionado un error que impedía eliminar descuentos de las líneas.
- **BUG. En el listado "Listado de líneas de albarán entre fechas de entrega" se muestran albaranes facturados cuando no lo están (ID: 8884168044):** Corregido un problema en el listado "Listado de líneas de albarán entre fechas de entrega" que mostraba albaranes facturados cuando no lo estaban.


## **N2** 


## **Novedades**

### Relacionado con tramitadores


### Desarrollos pagados por cliente


### Pequeños evolutivos


### Proyectos especiales

- **[PE.25.001] RAEE. INFORMES. Informe de historicos por código de etiquetas (ID: 7975905049):** Añadido un nuevo informe de históricos por código de etiquetas.
- **[PE.25.002] VERIFACTU. Bloque 1. Exportación registros de facturación [E16] (ID: 8816787071):** Implementada la exportación de registros de facturación.
- **[PE.25.002] VERIFACTU. Bloque 1. Guardado XML en S3 [E11] (ID: 8816790292):** Los XML de registro de facturación y registro de evento ahora se guardan en S3.
- **[PE.25.002] VERIFACTU. Bloque 1. Modelo registros de eventos [E3] [III] (ID: 8838736619):**  Se ha actualizado el modelo de registros de eventos.
- **[PE.25.002] VERIFACTU. Bloque 1. Análisis series para facturas rectificativas [A] (ID: 8851673176):** Se ha realizado un análisis de las series para facturas rectificativas.
- **[PE.25.002] VERIFACTU. Bloque 1. Creación de registros de facturación [E1] [IV] (ID: 8872179232):** Implementada la creación de registros de facturación.
- **[PE.25.002] VERIFACTU. Bloque 1. Modelo registros de eventos [E3] [IV] (ID: 8912876059):** Se ha actualizado el modelo de registros de eventos.


## **QA - Cobertura de test automáticos**


## **APS**


## **SYS y otros**


## **Desarrollos que afectan a la seguridad**


# **Validación en Sandbox**

## **Para paso a entorno de producción**

### Correcciones

| Ref. | Resp. | Comprobación | Quién N1 | N1 ok? | Quien QA? | QA ok? |
|---|---|---|---|---|---|---|
| m8817155664 |  | Comprobar que el informe "Resumen anual de entradas por centro" funciona correctamente. |  |  |  |  |
| m8874527315 |  | Comprobar que se pueden eliminar descuentos de las líneas sin errores. |  |  |  |  |
| m8884168044 |  | Comprobar que en el listado "Listado de líneas de albarán entre fechas de entrega" solo se muestran albaranes no facturados. |  |  |  |  |

### Novedades. En relación con las tramitaciones

| Ref. | Resp. | Comprobación | Quién N1 | N1 ok? | Quien QA? | QA ok? |
|---|---|---|---|---|---|---|
|  |  | NA |  |  |  |  |

### Novedades. Desarrollos pagados por cliente

| Ref. | Resp. | Comprobación | Quién N1 | N1 ok? | Quien QA? | QA ok? |
|---|---|---|---|---|---|---|
|  |  | NA |  |  |  |  |

### Novedades. Pequeños evolutivos

| Ref. | Resp. | Comprobación | Quién N1 | N1 ok? | Quien QA? | QA ok? |
|---|---|---|---|---|---|---|
|  |  | NA |  |  |  |  |

### Novedades. Proyectos especiales

| Ref. | Resp. | Comprobación | Quién N1 | N1 ok? | Quien QA? | QA ok? |
|---|---|---|---|---|---|---|
| 7975905049 |  | Comprobar que el nuevo informe de históricos por código de etiquetas funciona correctamente. |  |  |  |  |
| 8816787071 |  | Comprobar que la exportación de registros de facturación funciona correctamente. |  |  |  |  |
| 8816790292 |  | Comprobar que los XML se guardan correctamente en S3. |  |  |  |  |
| 8838736619 |  | Comprobar la correcta funcionalidad del modelo de registros de eventos. |  |  |  |  |
| 8851673176 |  | Comprobar la correcta funcionalidad del análisis de series para facturas rectificativas. |  |  |  |  |
| 8872179232 |  | Comprobar que la creación de registros de facturación funciona correctamente. |  |  |  |  |
| 8912876059 |  | Comprobar la correcta funcionalidad del modelo de registros de eventos. |  |  |  |  |


QA y APS

| Ref. | Resp. | Comprobación | Quién N1 | N1 ok? | Quien QA? | QA ok? |
|---|---|---|---|---|---|---|
|  |  | NA |  |  |  |  |



#  **Pruebas**

- TEST 1
- TEST 2
- Test 1
- Test 2
- Test 3
- No tests for now


# **Referencia commits**


---

### feat(8816787071): X [734d142]

feat(8816787071): X | Y | MONDAY TASKS: | - [PE.25.002] VERIFACTU. Bloque 1. Exportación registros de facturación [E16] (ID: 8816787071, URL: https://teimas.monday.com/boards/1013914950/pulses/8816787071) | BREAKING CHANGE: Z | Test Details: | - TEST 1 | - TEST 2 | Security: SEC 1 | SEC 2 | Refs: 8816787071

LudoBermejoES <LudoGithub@gmail.com> (Tue Apr 15 16:59:45 2025 +0200)

**Pruebas**:
- TEST 1
- TEST 2

**Seguridad**: SEC 1
 SEC 2

**Tareas relacionadas**:
- [PE.25.002] VERIFACTU. Bloque 1. Exportación registros de facturación [E16] (ID: 8816787071, Estado: active)

---

### feat(8817155664|8874527315|8884168044): Better template [4c7596]

feat(8817155664|8874527315|8884168044): Better template | Better template AI | MONDAY TASKS: | - BUG. Correción de informe "Resumen anual de entradas por centro" (ID: 8817155664, URL: https://teimas.monday.com/boards/1013914950/pulses/8817155664) | - BUG. Error al eliminar descuentos de líneas (ID: 8874527315, URL: https://teimas.monday.com/boards/1013914950/pulses/8874527315) | - BUG. En el listado "Listado de líneas de albarán entre fechas de entrega" se muestran albaranes facturados cuando no lo están (ID: 8884168044, URL: https://teimas.monday.com/boards/1013914950/pulses/8884168044) | Ticket de SupportBee: https://teimas.supportbee.com/tickets/51279217" | Security: NA | Refs: 8817155664|8874527315|8884168044

LudoBermejoES <LudoGithub@gmail.com> (Tue Apr 15 09:54:56 2025 +0200)

**Tareas relacionadas**:
- BUG. Correción de informe "Resumen anual de entradas por centro" (ID: 8817155664, Estado: active)
- BUG. Error al eliminar descuentos de líneas (ID: 8874527315, Estado: active)
- BUG. En el listado "Listado de líneas de albarán entre fechas de entrega" se muestran albaranes facturados cuando no lo están (ID: 8884168044, Estado: active)

---

### docs(8816790292|8912876059): Readme change [096edc]

docs(8816790292|8912876059): Readme change | Cambio en el readme | Test Details: | - TEST 1 | - TEST 2 | Security: Tened cuidado | Refs: 8816790292|8912876059 | MONDAY TASKS: | - [PE.25.002] VERIFACTU. Bloque 1. Guardado XML en S3 [E11] (ID: 8816790292, URL: https://teimas.monday.com/boards/1013914950/pulses/8816790292) | - [PE.25.002] VERIFACTU. Bloque 1. Modelo registros de eventos [E3] [IV] (ID: 8912876059, URL: https://teimas.monday.com/boards/1013914950/pulses/8912876059)

LudoBermejoES <LudoGithub@gmail.com> (Mon Apr 14 18:42:45 2025 +0200)

**Pruebas**:
- TEST 1
- TEST 2

**Seguridad**: Tened cuidado

**Tareas relacionadas**:
- [PE.25.002] VERIFACTU. Bloque 1. Guardado XML en S3 [E11] (ID: 8816790292, Estado: active)
- [PE.25.002] VERIFACTU. Bloque 1. Modelo registros de eventos [E3] [IV] (ID: 8912876059, Estado: active)


---

### docs(README): Fix readme [8522c7]

docs(README): Fix readme | Add all the new info to the readme | Security: NA

LudoBermejoES <LudoGithub@gmail.com> (Sun Apr 13 17:12:18 2025 +0200)

---

### feat(Use gemini ): use gemini from google to generate the release notes [27a55a]

feat(Use gemini ): use gemini from google to generate the release notes | To use it, I implemented everything to call 1.5-pro model | Security: NA

LudoBermejoES <LudoGithub@gmail.com> (Sun Apr 13 17:06:29 2025 +0200)

---

### feat(7975905049): Release-notes for gemini [910146]

feat(7975905049): Release-notes for gemini | First iteration of the release notes for gemini | Security: NA | Refs: 7975905049 | MONDAY TASKS: | - [PE.25.001] RAEE. INFORMES. Informe de historicos por código de etiquetas (ID: 7975905049, URL: https://teimas.monday.com/boards/1013914950/pulses/7975905049)

LudoBermejoES <LudoGithub@gmail.com> (Sun Apr 13 16:56:05 2025 +0200)

**Tareas relacionadas**:
- [PE.25.001] RAEE. INFORMES. Informe de historicos por código de etiquetas (ID: 7975905049, Estado: active)

---

### feat(8851673176|8872179232|8838736619): First version of release-notes [91a82d]

feat(8851673176|8872179232|8838736619): First version of release-notes | Release notes version with markdown prepared for gemini | Security: NA | Refs: 8851673176|8872179232|8838736619 | MONDAY TASKS: | - [PE.25.002] VERIFACTU. Bloque 1. Análisis series para facturas rectificativas [A] (ID: 8851673176, URL: https://teimas.monday.com/boards/1013914950/pulses/8851673176) | - [PE.25.002] VERIFACTU. Bloque 1. Creación de registros de facturación [E1] [IV] (ID: 8872179232, URL: https://teimas.monday.com/boards/1013914950/pulses/8872179232) | - [PE.25.002] VERIFACTU. Bloque 1. Modelo registros de eventos [E3] [III] (ID: 8838736619, URL: https://teimas.monday.com/boards/1013914950/pulses/8838736619)

LudoBermejoES <LudoGithub@gmail.com> (Sun Apr 13 15:47:49 2025 +0200)

**Tareas relacionadas**:
- [PE.25.002] VERIFACTU. Bloque 1. Análisis series para facturas rectificativas [A] (ID: 8851673176, Estado: active)
- [PE.25.002] VERIFACTU. Bloque 1. Creación de registros de facturación [E1] [IV] (ID: 8872179232, Estado: active)
- [PE.25.002] VERIFACTU. Bloque 1. Modelo registros de eventos [E3] [III] (ID: 8838736619, Estado: active)

---

### fix(NPM_REMOVAL): Remove NPM versioning [a570a6]

fix(NPM_REMOVAL): Remove NPM versioning | Remove npm versioning because we don't use it | Security: NA

LudoBermejoES <LudoGithub@gmail.com> (Sun Apr 13 13:27:59 2025 +0200)

---

### feat(8851673176|8872179232|8838736619): Improvements [5f0c72]

feat(8851673176|8872179232|8838736619): Improvements | Improvements with new lines | Test Details: | - Test 1 | - Test 2 | - Test 3 | Security: NA | Refs: 8851673176|8872179232|8838736619 | MONDAY TASKS: | - [PE.25.002] VERIFACTU. Bloque 1. Análisis series para facturas rectificativas [A] (ID: 8851673176, URL: https://teimas.monday.com/boards/1013914950/pulses/8851673176) | - [PE.25.002] VERIFACTU. Bloque 1. Creación de registros de facturación [E1] [IV] (ID: 8872179232, URL: https://teimas.monday.com/boards/1013914950/pulses/8872179232) | - [PE.25.002] VERIFACTU. Bloque 1. Modelo registros de eventos [E3] [III] (ID: 8838736619, URL: https://teimas.monday.com/boards/1013914950/pulses/8838736619)

LudoBermejoES <LudoGithub@gmail.com> (Sun Apr 13 13:20:05 2025 +0200)

**Pruebas**:
- Test 1
- Test 2
- Test 3

**Tareas relacionadas**:
- [PE.25.002] VERIFACTU. Bloque 1. Análisis series para facturas rectificativas [A] (ID: 8851673176, Estado: active)
- [PE.25.002] VERIFACTU. Bloque 1. Creación de registros de facturación [E1] [IV] (ID: 8872179232, Estado: active)
- [PE.25.002] VERIFACTU. Bloque 1. Modelo registros de eventos [E3] [III] (ID: 8838736619, Estado: active)

---

### refactor(Code): Better code [03041c]

refactor(Code): Better code | Code better | Security: NA

LudoBermejoES <LudoGithub@gmail.com> (Sun Apr 13 13:13:05 2025 +0200)

---

### feat(BM): Better monday [ac1aa4]

feat(BM): Better monday | Now we can add the scope with the ids of monday | Security: NA

LudoBermejoES <LudoGithub@gmail.com> (Sun Apr 13 12:39:13 2025 +0200)

---

### feat: Monday tasks 2 [c0f79d]

feat: Monday tasks 2 | Monday tasks | Security: NA

LudoBermejoES <LudoGithub@gmail.com> (Sun Apr 13 12:14:59 2025 +0200)


---

### feat(custom): Add url to search task [ce6fe6]

feat(custom): Add url to search task | search task with url so we can click and point to monday | security NA

LudoBermejoES <LudoGithub@gmail.com> (Sun Apr 13 10:35:14 2025 +0200)

---

### feat(custom): First version [104e18]

feat(custom): First version | First version of the system. It has loads of thing, but the main ones are related to the commit | message. I'll iterate over them in the next commits | testDetails No tests for now | security NA | references NA | BREAKING CHANGE: | First version, it is a breaking change

LudoBermejoES <LudoGithub@gmail.com> (Sun Apr 13 10:20:27 2025 +0200)

**Pruebas**:
- No tests for now
- security NA
- references NA

**Seguridad**: NA
references NA


---
