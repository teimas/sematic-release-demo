#!/usr/bin/env node
require('dotenv').config();
const mondaySdk = require('monday-sdk-js');
const monday = mondaySdk();
const prompts = require('prompts');
const fs = require('fs');
const path = require('path');

// Configura el token de API desde las variables de entorno
monday.setToken(process.env.MONDAY_API_KEY);
// Establece la versión de la API según la documentación
monday.setApiVersion("2024-10");

// Path to the temporary file where we'll store selected tasks
const tasksFilePath = path.join(__dirname, '..', '.monday-tasks-temp');

// Función para generar URL de Monday
const generateMondayUrl = (boardId, itemId) => {
  const template = process.env.MONDAY_URL_TEMPLATE || 
                  `https://${process.env.ACCOUNT_SLUG}.monday.com/boards/{board_id}/pulses/{item_id}`;
  return template
    .replace('{board_id}', boardId)
    .replace('{item_id}', itemId);
};

async function searchAndSelectTasks() {
  try {
    // Solicitar término de búsqueda
    const searchResponse = await prompts({
      type: 'text',
      name: 'searchTerm',
      message: 'Buscar tareas en Monday (Enter para omitir):',
    });

    // Si no se proporcionó un término de búsqueda, devolver cadena vacía
    if (!searchResponse.searchTerm || !searchResponse.searchTerm.trim()) {
      return '';
    }

    const boardId = process.env.MONDAY_BOARD_ID;
    let items = [];
    
    // Construir la consulta para buscar elementos que coincidan con el término de búsqueda
    let searchQuery;
    let variables = { limit: 20 };

    // Definir reglas de búsqueda basadas en los filtros
    let rules = [];
    
    // Añadir regla de búsqueda por texto
    if (searchResponse.searchTerm.trim()) {
      rules.push({
        column_id: "name",
        operator: "contains_text",
        compare_value: searchResponse.searchTerm
      });
    }
    
    // Definir parámetros de consulta
    const queryParams = rules.length > 0 ? 
      { rules: rules, operator: "and" } : undefined;
    
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
              board { id }
            }
          }
        }
      }`;
      variables.boardId = [boardId];
    } else {
      // Búsqueda global usando el endpoint tradicional
      searchQuery = `query ($limit: Int!, $queryParams: ItemsQuery) {
        items_page(limit: $limit, query_params: $queryParams) {
          cursor
          items {
            id
            name
            state
            board { id name }
          }
        }
      }`;
    }
    
    console.log('Buscando tareas en Monday...');
    const result = await monday.api(searchQuery, { variables });
    
    // Procesamiento según la estructura de respuesta
    if (boardId && result.data && result.data.boards && result.data.boards.length > 0) {
      // Caso de búsqueda en un tablero específico
      const board = result.data.boards[0];
      
      if (board.items_page && board.items_page.items) {
        items = board.items_page.items
          .filter(item => item.state === "active")
          .map(item => ({
            title: item.name,
            id: item.id,
            boardId: boardId,
            boardName: board.name,
            url: generateMondayUrl(boardId, item.id)
          }));
      }
    } else if (!boardId && result.data && result.data.items_page) {
      // Caso de búsqueda global
      items = result.data.items_page.items
        .filter(item => item.state === "active")
        .map(item => ({
          title: item.name,
          id: item.id,
          boardId: item.board?.id,
          boardName: item.board?.name,
          url: generateMondayUrl(item.board?.id, item.id)
        }));
    }
    
    if (items.length === 0) {
      console.log('No se encontraron tareas que coincidan con los criterios de búsqueda.');
      return '';
    }
    
    // Preparar opciones para selección múltiple
    const choices = items.map(item => ({
      title: `${item.title} (ID: ${item.id})`,
      value: item
    }));
    
    // Permitir selección múltiple de tareas
    const selectedResponse = await prompts({
      type: 'multiselect',
      name: 'selectedTasks',
      message: 'Selecciona las tareas resueltas por este commit:',
      choices: choices,
      hint: '- Espacio para seleccionar, Enter para confirmar'
    });
    
    if (!selectedResponse.selectedTasks || selectedResponse.selectedTasks.length === 0) {
      return '';
    }
    
    // Formatear las tareas seleccionadas para el mensaje de commit
    const formattedTasks = selectedResponse.selectedTasks.map(task => 
      `${task.title} (ID: ${task.id}, URL: ${task.url})`
    ).join(', ');
    
    return formattedTasks;
  } catch (error) {
    console.error('Error al buscar tareas:', error.message);
    if (error.data) {
      console.error('Detalles del error:', JSON.stringify(error.data, null, 2));
    }
    return '';
  }
}

// Ejecutar la función principal y guardar el resultado en un archivo temporal
searchAndSelectTasks()
  .then(result => {
    if (result) {
      console.log('\nTareas seleccionadas:');
      console.log(result);
      console.log('\nEstas tareas se incluirán en el mensaje de commit.');
      
      // Save selected tasks to a temporary file
      fs.writeFileSync(tasksFilePath, result);
    } else {
      console.log('No se seleccionaron tareas.');
      // Ensure the file is empty if no tasks are selected
      if (fs.existsSync(tasksFilePath)) {
        fs.writeFileSync(tasksFilePath, '');
      }
    }
    process.exit(0);
  })
  .catch(error => {
    console.error('Error inesperado:', error);
    process.exit(1);
  }); 
