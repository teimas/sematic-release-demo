# Actualización Teixo versión tag-teixo-20250410-1.111.0

# **Información para N1**

Tickets que se solucionan en esta actualización

| Desarrollo | IDTarea | Support Bee |
| ----- | ----- | ----- |
|  | m8840639326 | https://teimas.supportbee.com/tickets/51272047 |
|  | m8805697272 | https://teimas.supportbee.com/tickets/51289664 |
|  | m8819524238 | https://teimas.supportbee.com/tickets/51316560 |
|  | m8817923813 | https://teimas.supportbee.com/tickets/49751920 |

# **Información técnica**

### Responsable despliegue

[Andrés Pedraza de la Cuesta](mailto:andres.pedraza@teimas.com)

### Etiquetas

### Migraciones lentas

| IDTarea | Fichero | Tiempos |
| ----- | ----- | ----- |
|  |  |  |

### Partes a ejecutar

| IDTarea | Enlace a Script |
| ----- | ----- |
| m8392481017 | https://redmine.teimas.com/issues/35728 |

## 

## **Cambios para entorno de producción**

## **Correcciones**

AQUI TIENES QUE METER TODAS LAS TAREAS DE MONDAY QUE TENGAN LA LABEL "1. BUG". A CONTINUACIÓN DOS EJEMPLOS DE FORMATO:

## **N2** 

AQUI TIENES QUE METER TODAS LAS TAREAS DE MONDAY QUE TENGAN LA LABEL "N2". A CONTINUACIÓN DOS EJEMPLOS DE FORMATO:

## **Novedades**

### Relacionado con tramitadores

AQUI TIENES QUE METER TODAS LAS TAREAS DE MONDAY QUE TENGAN LA LABEL "2. TRAMITADORES". A CONTINUACIÓN DOS EJEMPLOS DE FORMATO:

### Desarrollos pagados por cliente

AQUI TIENES QUE METER TODAS LAS TAREAS DE MONDAY QUE TENGAN LA LABEL "3. CLIENTE - PAGADO". A CONTINUACIÓN DOS EJEMPLOS DE FORMATO:

### Pequeños evolutivos

AQUI TIENES QUE METER TODAS LAS TAREAS DE MONDAY QUE TENGAN LA LABEL "4. EVOLUTIVO". A CONTINUACIÓN DOS EJEMPLOS DE FORMATO:

### Proyectos especiales

AQUI TIENES QUE METER TODAS LAS TAREAS DE MONDAY QUE TENGAN LA LABEL "PE". A CONTINUACIÓN DOS EJEMPLOS DE FORMATO:

## **QA \- Cobertura de test automáticos**

AQUI TIENES QUE METER TODAS LAS TAREAS DE MONDAY QUE TENGAN LA LABEL "QA". A CONTINUACIÓN DOS EJEMPLOS DE FORMATO:

## **APS**

AQUI TIENES QUE METER TODAS LAS TAREAS DE MONDAY QUE TENGAN LA LABEL "APS". A CONTINUACIÓN DOS EJEMPLOS DE FORMATO:

## **SYS y otros**

AQUI TIENES QUE METER TODAS LAS TAREAS DE MONDAY QUE TENGAN LA LABEL "SYS". A CONTINUACIÓN DOS EJEMPLOS DE FORMATO:

## **Desarrollos que afectan a la seguridad**

AQUI TIENES QUE METER TODAS LAS TAREAS DE MONDAY QUE TENGAN LA LABEL "SEC". A CONTINUACIÓN DOS EJEMPLOS DE FORMATO:

# **Validación en Sandbox**

## **Para paso a entorno de producción**

### Correcciones

| Ref. | Resp. | Comprobación | Quién N1 | N1 ok? | Quien QA? | QA ok? |
| ----- | ----- | ----- | ----- | ----- | ----- | ----- |
| m8840639326 | FV | En el apartado de Mis informes, generar un informe de "Recogidas entre fechas detallado". En dicho informe seleccionar como fecha de inicio y fecha de fin una misma fecha, que tenga recogidas. En elementos de filtrado, filtrar primeramente por estados "Notificación" y el estado que tengan esas recogidas. Posteriormente, filtrar únicamente por el estado que tengan esas recogidas. Comprobar que en ambos casos se muestran los mismo datos. | **CS** | **OK** |  |  |
| m8817923813 | FV | Para un DI indicar distintos datos de tabla 5 y características de peligrosidad en el residuo del productor y el residuo del gestor. Comprobar que al descargar el xml 3.5 del DI, en el apartado de "DCSProducerResidueIdentification" aparecen los datos del residuo del gestor, en lugar de los del productor. Para una NT indicar distintos datos de tabla 5 y características de peligrosidad en el residuo del productor y el residuo del gestor. Comprobar que al descargar el xml 3.5 de la NT, en el apartado de "NTResidueIdentification" se muestran los datos del residuo del gestor, en lugar de los datos del residuo del productor. | **CS** | **1.OK 2.OK**  |  |  |

### Novedades. En relación con las tramitaciones

| Ref. | Resp. | Comprobación | Quién N1 | N1 ok? | Quien QA? | QA ok? |
| ----- | ----- | ----- | ----- | ----- | ----- | ----- |
|  |  | NA |  |  |  |  |

### Novedades. Desarrollos pagados por cliente

| Ref. | Resp. | Comprobación | Quién N1 | N1 ok? | Quien QA? | QA ok? |
| ----- | ----- | ----- | ----- | ----- | ----- | ----- |
|  |  | NA |  |  |  |  |

### Novedades. Pequeños evolutivos

| Ref. | Resp. | Comprobación | Quién N1 | N1 ok? | Quien QA? | QA ok? |
| ----- | ----- | ----- | ----- | ----- | ----- | ----- |
| m8739853753 | SG | Acceder a una cuenta que pueda importar entradas. Generamos un archivo CSV de importación que contenga un residuo que esté asociado al centro productor. Importar una entrada desde CSV. Comprobamos que el residuo asociado a la entrada creada, es el residuo del centro productor. | **CS** | **OK** |  |  |
| m8805697272 | SG | Se puede usar de ejemplo este tiket [https://teimas.supportbee.com/tickets/51289664](https://teimas.supportbee.com/tickets/51289664) En una cuenta que utilice SRAPS. Seleccionamos un centro SRAP cualquiera, y si no tiene una autorización deltipo SCR  o SCI se las generamos. Generamos una recogida con una o varias lineas(con 1 es suficiente). Añadimos el centro SRAP con la autorización SCR/SCI y generamos el DI desde la entrada. Comprobamos que sale la autorización correcta. | CS | OK |  |  |

### Novedades. Proyectos especiales

| Ref. | Resp. | Comprobación | Quién N1 | N1 ok? | Quien QA? | QA ok? |
| ----- | ----- | ----- | ----- | ----- | ----- | ----- |
|  |  | NA |  |  |  |  |

   
QA y APS

| Ref. | Resp. | Comprobación | Quién N1 | N1 ok? | Quien QA? | QA ok? |
| ----- | ----- | ----- | ----- | ----- | ----- | ----- |
|  |  | NA |  |  |  |  |

   

#  **Pruebas**

Aquí vendrán todos los tests que están marcados en CADA UNO DE LOS COMMITS. No dejes ninguno fuera. Todos y cada uno de ellos.

# **Referencia commits**

Aquí irán absolutamente TODOS los commits que recibas. No dejes ninguno. Exactamente como tienes en el documento de entrada.