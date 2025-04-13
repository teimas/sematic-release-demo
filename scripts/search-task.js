#!/usr/bin/env node
require('dotenv').config();
const mondaySdk = require('monday-sdk-js');
const monday = mondaySdk();
const prompts = require('prompts');

// Configura el token de API desde las variables de entorno
monday.setToken(process.env.MONDAY_API_KEY);
// Establece la versión de la API según la documentación (última versión disponible)
monday.setApiVersion("2024-10");

async function searchTasks() {
  // Solicitar término de búsqueda
  const searchResponse = await prompts({
    type: 'text',
    name: 'searchTerm',
    message: 'Ingresa el término de búsqueda para tareas (opcional, Enter para omitir):',
  });

  // Preguntar si desea filtrar por estado
  const filterByStatus = await prompts({
    type: 'confirm',
    name: 'value',
    message: '¿Deseas filtrar por estado (ej. "Hecho", "En progreso")?',
    initial: false
  });

  let statusFilter = null;
  if (filterByStatus.value) {
    const statusResponse = await prompts({
      type: 'text',
      name: 'statusValue',
      message: 'Ingresa el estado que deseas filtrar:',
    });
    if (statusResponse.statusValue) {
      statusFilter = statusResponse.statusValue;
    }
  }

  const boardId = process.env.MONDAY_BOARD_ID;
  
  // Si no hay un ID de tablero configurado, primero buscaremos los tableros
  if (!boardId) {
    console.log('Buscando en todos los tableros accesibles...');
    
    try {
      // Obtener los tableros a los que tiene acceso el usuario
      const boardsQuery = `query { 
        boards (limit: 10) { 
          id 
          name 
          description 
          state 
        } 
      }`;
      
      const result = await monday.api(boardsQuery);
      
      if (result.data && result.data.boards) {
        console.log('Tableros encontrados:');
        const activeBoards = result.data.boards.filter(board => board.state === "active");
        
        if (activeBoards.length === 0) {
          console.log('No se encontraron tableros activos.');
        } else {
          activeBoards.forEach(board => {
            console.log(`- ${board.name} (ID: ${board.id})`);
            if (board.description) {
              console.log(`  Descripción: ${board.description}`);
            }
          });
        }
        
        console.log('\nPara mejorar la búsqueda, configura un ID de tablero específico ejecutando "npm run config"');
      }
    } catch (error) {
      console.error('Error al buscar tableros:', error.message);
      if (error.data) {
        console.error('Detalles del error:', JSON.stringify(error.data, null, 2));
      }
    }
  }
  
  try {
    // Construir la consulta para buscar elementos que coincidan con el término de búsqueda
    let searchQuery;
    let variables = { 
      limit: 20
    };

    // Definir reglas de búsqueda basadas en los filtros
    let rules = [];
    let operator = "and";
    
    // Añadir regla de búsqueda por texto si se proporcionó
    if (searchResponse.searchTerm) {
      variables.searchTerm = searchResponse.searchTerm;
      // Para búsqueda por texto, usamos rules en lugar de query
      if (searchResponse.searchTerm.trim()) {
        rules.push({
          column_id: "name",
          operator: "contains_text",
          compare_value: searchResponse.searchTerm
        });
      }
    }
    
    // Añadir regla de filtro por estado si se proporcionó
    if (statusFilter) {
      rules.push({
        column_id: "status",
        compare_value: statusFilter
      });
    }
    
    // Definir parámetros de consulta
    const queryParams = rules.length > 0 ? 
      { rules: rules, operator: operator } : 
      undefined;
    
    if (queryParams) {
      variables.queryParams = queryParams;
    }
    
    if (boardId) {
      // Búsqueda en un tablero específico usando el ID del tablero guardado
      searchQuery = `query ($boardId: [ID!], $limit: Int!, $queryParams: ItemsQuery) {
        boards(ids: $boardId) {
          name
          items_page(limit: $limit, query_params: $queryParams) {
            cursor
            items {
              id
              name
              state
              updated_at
              group {
                id
                title
              }
              column_values {
                id
                text
                value
                type
              }
              creator {
                id
                name
              }
            }
          }
        }
      }`;
      variables.boardId = [boardId]; // Ya no necesitamos convertirlo a Int
    } else {
      // Búsqueda global usando el endpoint tradicional
      searchQuery = `query ($limit: Int!, $queryParams: ItemsQuery) {
        items_page(limit: $limit, query_params: $queryParams) {
          cursor
          items {
            id
            name
            state
            updated_at
            board {
              id
              name
            }
            group {
              id
              title
            }
            creator {
              id
              name
            }
          }
        }
      }`;
    }
    
    console.log('Realizando búsqueda...' + searchQuery, JSON.stringify(variables, null, 2));
    const result = await monday.api(searchQuery, { variables });
    
    // Procesamiento según la estructura de respuesta
    if (boardId && result.data && result.data.boards && result.data.boards.length > 0) {
      // Caso de búsqueda en un tablero específico
      const board = result.data.boards[0];
      console.log(`\nBuscando en tablero: ${board.name}`);
      
      if (board.items_page && board.items_page.items) {
        const items = board.items_page.items.filter(item => item.state === "active");
        
        if (items.length === 0) {
          console.log('No se encontraron tareas activas que coincidan con los criterios de búsqueda.');
        } else {
          console.log(`\nTareas encontradas (${items.length}):`);
          items.forEach(item => {
            console.log(`- ${item.name} (ID: ${item.id})`);
            console.log(`  Actualizado: ${new Date(item.updated_at).toLocaleString()}`);
            console.log(`  Creado por: ${item.creator?.name || 'N/A'}`);
            console.log(`  Grupo: ${item.group?.title || 'N/A'}`);
            
            console.log('  Detalles:');
            if (item.column_values && item.column_values.length > 0) {
              const visibleColumns = item.column_values.filter(col => col.text && col.text.trim() !== '');
              if (visibleColumns.length > 0) {
                visibleColumns.forEach(col => {
                  // Usamos solo id y text ya que title no está disponible
                  console.log(`    ID columna ${col.id}: ${col.text}`);
                });
              } else {
                console.log('    No hay detalles adicionales disponibles');
              }
            } else {
              console.log('    No hay detalles adicionales disponibles');
            }
            
            console.log('');
          });
          
          if (board.items_page.cursor) {
            console.log('Hay más resultados disponibles. Refina tu búsqueda para ver resultados más específicos.');
          }
        }
      } else {
        console.log('No se encontraron elementos en el tablero.');
      }
    } else if (!boardId && result.data && result.data.items_page) {
      // Caso de búsqueda global
      const items = result.data.items_page.items.filter(item => item.state === "active");
      
      if (items.length === 0) {
        console.log('No se encontraron tareas activas que coincidan con los criterios de búsqueda.');
      } else {
        console.log(`\nTareas encontradas (${items.length}):`);
        items.forEach(item => {
          console.log(`- ${item.name} (ID: ${item.id})`);
          console.log(`  Actualizado: ${new Date(item.updated_at).toLocaleString()}`);
          console.log(`  Creado por: ${item.creator?.name || 'N/A'}`);
          console.log(`  Tablero: ${item.board?.name} (ID: ${item.board?.id})`);
          console.log(`  Grupo: ${item.group?.title || 'N/A'}`);
          console.log('');
        });
        
        if (result.data.items_page.cursor) {
          console.log('Hay más resultados disponibles. Refina tu búsqueda para ver resultados más específicos.');
        }
      }
    } else {
      console.log('No se pudieron obtener resultados de búsqueda.');
      console.log('Respuesta de la API:', JSON.stringify(result, null, 2));
    }
  } catch (error) {
    console.error('Error al buscar tareas:', error.message);
    if (error.data) {
      console.error('Detalles del error:', JSON.stringify(error.data, null, 2));
    }
    
    // Si el error indica un problema de autenticación, sugerir volver a configurar
    if (error.message.includes('Authentication') || error.message.includes('token')) {
      console.log('\n⚠️ Parece que hay un problema con la autenticación.');
      console.log('Ejecuta "npm run config" para actualizar tu token de API de Monday.com.');
    }
  }
}

searchTasks().catch(error => {
  console.error('Error inesperado:', error);
});