# Actualización Teixo versión tag-teixo-20250410-1.111.0

# **Información para N1**

Tickets que se solucionan en esta actualización

| Desarrollo | IDTarea | Support Bee |
|---|---|---|
| BUG. Correción de informe "Resumen anual de entradas por centro" (ID: 8817155664) | 8817155664 | https://teimas.supportbee.com/tickets/51279217 |
| BUG. Error al eliminar descuentos de líneas (ID: 8874527315) | 8874527315 | https://teimas.supportbee.com/tickets/51385685, https://teimas.supportbee.com/tickets/51414391, https://teimas.supportbee.com/tickets/51366346 |
| BUG. En el listado "Listado de líneas de albarán entre fechas de entrega" se muestran albaranes facturados cuando no lo están (ID: 8884168044) | 8884168044 | https://teimas.supportbee.com/tickets/51429571 |


# **Información técnica**

### Responsable despliegue

Aquí el nombre de la persona que lanzó la petición

### Etiquetas

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

- **BUG. Correción de informe "Resumen anual de entradas por centro" (ID: 8817155664):**  Se ha corregido un error en el informe "Resumen anual de entradas por centro".
- **BUG. Error al eliminar descuentos de líneas (ID: 8874527315):** Se ha solucionado un problema que impedía eliminar descuentos de las líneas.
- **BUG. En el listado "Listado de líneas de albarán entre fechas de entrega" se muestran albaranes facturados cuando no lo están (ID: 8884168044):** Se ha corregido un error en el listado "Listado de líneas de albarán entre fechas de entrega" que mostraba albaranes facturados cuando no lo estaban.


## **N2** 

## **Novedades**

### Relacionado con tramitadores


### Desarrollos pagados por cliente


### Pequeños evolutivos

### Proyectos especiales

- **[PE.25.001] RAEE. INFORMES. Informe de historicos por código de etiquetas (ID: 7975905049):** Se ha añadido un nuevo informe de históricos por código de etiquetas.
- **[PE.25.002] VERIFACTU. Bloque 1. Guardado XML en S3 [E11] (ID: 8816790292):** Se ha implementado el guardado de XML de registro de facturación y registro de evento en S3.
- **[PE.25.002] VERIFACTU. Bloque 1. Modelo registros de eventos [E3] [III] (ID: 8838736619):** Se ha actualizado el modelo de registros de eventos.
- **[PE.25.002] VERIFACTU. Bloque 1. Análisis series para facturas rectificativas [A] (ID: 8851673176):** Se ha realizado un análisis de las series para facturas rectificativas.
- **[PE.25.002] VERIFACTU. Bloque 1. Creación de registros de facturación [E1] [IV] (ID: 8872179232):** Se ha implementado la creación de registros de facturación.
- **[PE.25.002] VERIFACTU. Bloque 1. Modelo registros de eventos [E3] [IV] (ID: 8912876059):** Se ha actualizado el modelo de registros de eventos.


## **QA \- Cobertura de test automáticos**

## **APS**

## **SYS y otros**

## **Desarrollos que afectan a la seguridad**


# **Validación en Sandbox**

## **Para paso a entorno de producción**

### Correcciones

| Ref. | Resp. | Comprobación | Quién N1 | N1 ok? | Quien QA? | QA ok? |
|---|---|---|---|---|---|---|
| 8817155664 |  | Verificar la correcta generación del informe "Resumen anual de entradas por centro" |  |  |  |  |
| 8874527315 |  | Comprobar que se pueden eliminar descuentos de las líneas sin errores. |  |  |  |  |
| 8884168044 |  | Verificar que en el listado "Listado de líneas de albarán entre fechas de entrega" solo se muestran los albaranes que no están facturados. |  |  |  |  |


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


### Novedades. Proyectos especiales

| Ref. | Resp. | Comprobación | Quién N1 | N1 ok? | Quien QA? | QA ok? |
|---|---|---|---|---|---|---|
| 7975905049 |  | Verificar la correcta generación del informe de históricos por código de etiquetas. |  |  |  |  |
| 8816790292 |  | Comprobar que los XML de registro de facturación y registro de evento se guardan correctamente en S3. |  |  |  |  |
| 8838736619 |  | Validar el funcionamiento del nuevo modelo de registros de eventos. |  |  |  |  |
| 8851673176 |  |  Comprobar el correcto análisis de series para facturas rectificativas. |  |  |  |  |
| 8872179232 |  | Verificar la correcta creación de registros de facturación. |  |  |  |  |
| 8912876059 |  | Validar el funcionamiento del nuevo modelo de registros de eventos. |  |  |  |  |



QA y APS

| Ref. | Resp. | Comprobación | Quién N1 | N1 ok? | Quien QA? | QA ok? |
|---|---|---|---|---|---|---|



#  **Pruebas**

- TEST 1
- TEST 2
- Test 1
- Test 2
- Test 3
- No tests for now


# **Referencia commits**


---

### feat(8817155664|8874527315|8884168044): Better template [4c75960]

feat(8817155664|8874527315|8884168044): Better template | Better template AI | MONDAY TASKS: | - BUG. Correción de informe "Resumen anual de entradas por centro" (ID: 8817155664, URL: https://teimas.monday.com/boards/1013914950/pulses/8817155664) | - BUG. Error al eliminar descuentos de líneas (ID: 8874527315, URL: https://teimas.monday.com/boards/1013914950/pulses/8874527315) | - BUG. En el listado "Listado de líneas de albarán entre fechas de entrega" se muestran albaranes facturados cuando no lo están (ID: 8884168044, URL: https://teimas.monday.com/boards/1013914950/pulses/8884168044) | Ticket de SupportBee: https://teimas.supportbee.com/tickets/51279217" | Security: NA | Refs: 8817155664|8874527315|8884168044

**Tareas relacionadas**:
- BUG. Correción de informe "Resumen anual de entradas por centro" (ID: 8817155664, Estado: active)
- BUG. Error al eliminar descuentos de líneas (ID: 8874527315, Estado: active)
- BUG. En el listado "Listado de líneas de albarán entre fechas de entrega" se muestran albaranes facturados cuando no lo están (ID: 8884168044, Estado: active)

---

### docs(8816790292|8912876059): Readme change [096edc]

docs(8816790292|8912876059): Readme change | Cambio en el readme | Test Details: | - TEST 1 | - TEST 2 | Security: Tened cuidado | Refs: 8816790292|8912876059 | MONDAY TASKS: | - [PE.25.002] VERIFACTU. Bloque 1. Guardado XML en S3 [E11] (ID: 8816790292, URL: https://teimas.monday.com/boards/1013914950/pulses/8816790292) | - [PE.25.002] VERIFACTU. Bloque 1. Modelo registros de eventos [E3] [IV] (ID: 8912876059, URL: https://teimas.monday.com/boards/1013914950/pulses/8912876059)

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

---

### feat(Use gemini ): use gemini from google to generate the release notes [27a55a]

feat(Use gemini ): use gemini from google to generate the release notes | To use it, I implemented everything to call 1.5-pro model | Security: NA

---

### feat(7975905049): Release-notes for gemini [910146]

feat(7975905049): Release-notes for gemini | First iteration of the release notes for gemini | Security: NA | Refs: 7975905049 | MONDAY TASKS: | - [PE.25.001] RAEE. INFORMES. Informe de historicos por código de etiquetas (ID: 7975905049, URL: https://teimas.monday.com/boards/1013914950/pulses/7975905049)

**Tareas relacionadas**:
- [PE.25.001] RAEE. INFORMES. Informe de historicos por código de etiquetas (ID: 7975905049, Estado: active)

---

### feat(8851673176|8872179232|8838736619): First version of release-notes [91a82d]

feat(8851673176|8872179232|8838736619): First version of release-notes | Release notes version with markdown prepared for gemini | Security: NA | Refs: 8851673176|8872179232|8838736619 | MONDAY TASKS: | - [PE.25.002] VERIFACTU. Bloque 1. Análisis series para facturas rectificativas [A] (ID: 8851673176, URL: https://teimas.monday.com/boards/1013914950/pulses/8851673176) | - [PE.25.002] VERIFACTU. Bloque 1. Creación de registros de facturación [E1] [IV] (ID: 8872179232, URL: https://teimas.monday.com/boards/1013914950/pulses/8872179232) | - [PE.25.002] VERIFACTU. Bloque 1. Modelo registros de eventos [E3] [III] (ID: 8838736619, URL: https://teimas.monday.com/boards/1013914950/pulses/8838736619)

**Tareas relacionadas**:
- [PE.25.002] VERIFACTU. Bloque 1. Análisis series para facturas rectificativas [A] (ID: 8851673176, Estado: active)
- [PE.25.002] VERIFACTU. Bloque 1. Creación de registros de facturación [E1] [IV] (ID: 8872179232, Estado: active)
- [PE.25.002] VERIFACTU. Bloque 1. Modelo registros de eventos [E3] [III] (ID: 8838736619, Estado: active)

---

### fix(NPM_REMOVAL): Remove NPM versioning [a570a6]

fix(NPM_REMOVAL): Remove NPM versioning | Remove npm versioning because we don't use it | Security: NA

---

### feat(8851673176|8872179232|8838736619): Improvements [5f0c72]

feat(8851673176|8872179232|8838736619): Improvements | Improvements with new lines | Test Details: | - Test 1 | - Test 2 | - Test 3 | Security: NA | Refs: 8851673176|8872179232|8838736619 | MONDAY TASKS: | - [PE.25.002] VERIFACTU. Bloque 1. Análisis series para facturas rectificativas [A] (ID: 8851673176, URL: https://teimas.monday.com/boards/1013914950/pulses/8851673176) | - [PE.25.002] VERIFACTU. Bloque 1. Creación de registros de facturación [E1] [IV] (ID: 8872179232, URL: https://teimas.monday.com/boards/1013914950/pulses/8872179232) | - [PE.25.002] VERIFACTU. Bloque 1. Modelo registros de eventos [E3] [III] (ID: 8838736619, URL: https://teimas.monday.com/boards/1013914950/pulses/8838736619)


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

---

### feat(BM): Better monday [ac1aa4]

feat(BM): Better monday | Now we can add the scope with the ids of monday | Security: NA

---

### feat: Monday tasks 2 [c0f79d]

feat: Monday tasks 2 | Monday tasks | Security: NA

---

### feat(custom): Add url to search task [ce6fe6]

feat(custom): Add url to search task | search task with url so we can click and point to monday | security NA

---

### feat(custom): First version [104e18]

feat(custom): First version | First version of the system. It has loads of thing, but the main ones are related to the commit | message. I'll iterate over them in the next commits | testDetails No tests for now | security NA | references NA | BREAKING CHANGE: | First version, it is a breaking change

**Pruebas**:
- No tests for now

**Seguridad**: NA


---

