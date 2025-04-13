#!/usr/bin/env node
require('dotenv').config();
const mondaySdk = require('monday-sdk-js');
const monday = mondaySdk();
const prompts = require('prompts');
const { spawn, execSync } = require('child_process');
const fs = require('fs');
const path = require('path');

// Configura el token de API desde las variables de entorno
monday.setToken(process.env.MONDAY_API_KEY);
// Establece la versión de la API según la documentación
monday.setApiVersion("2024-10");

// Función para generar URL de Monday
const generateMondayUrl = (boardId, itemId) => {
  const template = process.env.MONDAY_URL_TEMPLATE || 
                  `https://${process.env.ACCOUNT_SLUG}.monday.com/boards/{board_id}/pulses/{item_id}`;
  return template
    .replace('{board_id}', boardId)
    .replace('{item_id}', itemId);
};

// Definición de tipos de commit
const commitTypes = [
  { value: 'feat', name: 'feat:     A new feature' },
  { value: 'fix', name: 'fix:      A bug fix' },
  { value: 'docs', name: 'docs:     Documentation only changes' },
  { value: 'style', name: 'style:    Code style changes (formatting, etc)' },
  { value: 'refactor', name: 'refactor: Code changes that neither fix bugs nor add features' },
  { value: 'perf', name: 'perf:     Performance improvements' },
  { value: 'test', name: 'test:     Adding or fixing tests' },
  { value: 'chore', name: 'chore:    Changes to the build process or auxiliary tools' },
  { value: 'revert', name: 'revert:   Revert to a commit' }
];

// Función auxiliar para obtener los IDs de las tareas seleccionadas
function getTaskIdsFromSelection(tasks) {
  if (!tasks || tasks.length === 0) {
    return '';
  }
  return tasks.map(task => task.id).join('|');
}

// Función para construir la consulta de búsqueda de tareas
function buildSearchQuery(boardId) {
  if (boardId) {
    // Búsqueda en un tablero específico
    return `query ($boardId: [ID!], $limit: Int!, $queryParams: ItemsQuery) {
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
  } else {
    // Búsqueda global
    return `query ($limit: Int!, $queryParams: ItemsQuery) {
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
}

// Función para construir las variables de búsqueda
function buildSearchVariables(searchTerm, boardId) {
  const variables = { limit: 20 };
  
  // Añadir regla de búsqueda por texto
  const rules = [
    {
      column_id: "name",
      operator: "contains_text",
      compare_value: searchTerm
    }
  ];
  
  // Definir parámetros de consulta
  variables.queryParams = { rules: rules, operator: "and" };
  
  // Añadir boardId si está disponible
  if (boardId) {
    variables.boardId = [boardId];
  }
  
  return variables;
}

// Función para realizar la búsqueda en Monday
async function searchMondayTasks(searchTerm) {
  try {
    const boardId = process.env.MONDAY_BOARD_ID;
    
    // Construir consulta y variables
    const searchQuery = buildSearchQuery(boardId);
    const variables = buildSearchVariables(searchTerm, boardId);
    
    console.log('Buscando tareas en Monday...');
    const result = await monday.api(searchQuery, { variables });
    
    return processSearchResults(result, boardId);
  } catch (error) {
    console.error('Error al buscar tareas en Monday:', error.message);
    return [];
  }
}

// Función para procesar los resultados de búsqueda
function processSearchResults(result, boardId) {
  let items = [];
  
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
  
  return items;
}

// Función para permitir al usuario seleccionar tareas
async function selectMondayTasks(tasks) {
  if (tasks.length === 0) {
    console.log('No se encontraron tareas que coincidan con los criterios de búsqueda.');
    return [];
  }
  
  // Preparar opciones para selección múltiple
  const choices = tasks.map(item => ({
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
  
  return selectedResponse.selectedTasks || [];
}

// Función para formatear las tareas para el mensaje de commit
function formatSelectedTasks(tasks) {
  if (!tasks || tasks.length === 0) {
    return '';
  }
  
  return tasks.map(task => 
    `${task.title} (ID: ${task.id}, URL: ${task.url})`
  ).join(', ');
}

async function createCommit() {
  try {
    // Variable para almacenar el scope
    let scope = '';
    
    // Variable para almacenar las tareas de Monday
    let mondayTasks = '';
    
    // Variable para almacenar la referencia de tickets
    let ticketReference = '';
    
    // Variable para almacenar las tareas seleccionadas
    let selectedMondayTasks = [];
    
    // Solicitar término de búsqueda para Monday
    const searchResponse = await prompts({
      type: 'text',
      name: 'searchTerm',
      message: 'Buscar tareas en Monday (Enter para omitir):',
    });
  
    if (searchResponse.searchTerm && searchResponse.searchTerm.trim()) {
      // Buscar tareas en Monday
      const items = await searchMondayTasks(searchResponse.searchTerm.trim());
      
      // Permitir selección de tareas
      selectedMondayTasks = await selectMondayTasks(items);
      
      if (selectedMondayTasks.length > 0) {
        // Formatear las tareas seleccionadas para el mensaje de commit
        mondayTasks = formatSelectedTasks(selectedMondayTasks);
        
        console.log('\nTareas seleccionadas:');
        console.log(mondayTasks);
        
        // Extraer scope e IDs para referencias de tickets
        if (selectedMondayTasks.length > 0) {
          // Usar los IDs de las tareas como scope
          scope = getTaskIdsFromSelection(selectedMondayTasks);
          console.log(`Scope generado automáticamente de IDs de tareas: ${scope}`);
          
          // Usar los mismos IDs para la referencia de tickets
          ticketReference = scope;
          console.log(`Referencia de tickets generada automáticamente: ${ticketReference}`);
        }
      }
    }

    // Iniciar información de commit
    console.log('\n--- Información del commit ---');
    
    // Preguntar por tipo de commit
    const typeResponse = await prompts({
      type: 'select',
      name: 'type',
      message: 'Select the TYPE of change:',
      choices: commitTypes,
      initial: 0
    });
    
    // Preguntar por scope solo si no se pudo extraer de las tareas de Monday
    if (!scope) {
      const scopeResponse = await prompts({
        type: 'text',
        name: 'scope',
        message: 'Enter the SCOPE (PE.XX.XXX format):',
      });
      scope = scopeResponse.scope || '';
    }
    
    // Preguntar por título/subject
    const subjectResponse = await prompts({
      type: 'text',
      name: 'subject',
      message: 'Enter a SHORT title:',
      validate: value => value.length > 0 ? true : 'El título es obligatorio'
    });
    
    // Preguntar por descripción
    const bodyResponse = await prompts({
      type: 'text',
      name: 'body',
      message: 'Enter a DETAILED description (optional):'
    });
    
    // Preguntar por breaking changes
    const breakingResponse = await prompts({
      type: 'text',
      name: 'breaking',
      message: 'List any BREAKING CHANGES (optional):'
    });
    
    // Preguntar por test details
    const testDetailsResponse = await prompts({
      type: 'text',
      name: 'testDetails',
      message: 'Enter TEST details (optional, use | for new lines):'
    });
    
    // Preguntar por seguridad
    const securityResponse = await prompts({
      type: 'text',
      name: 'security',
      message: 'Enter SECURITY considerations (use | for new lines, NA if not applicable):'
    });
    
    // Preguntar por ticket reference solo si no hay tareas seleccionadas
    if (!ticketReference) {
      const referencesResponse = await prompts({
        type: 'text',
        name: 'references',
        message: 'Enter ticket reference (mXXXXXXXXXX format):',
      });
      ticketReference = referencesResponse.references || '';
    }
    
    // Preguntar por change ID
    const changeIdResponse = await prompts({
      type: 'text',
      name: 'changeId',
      message: 'Enter Change-Id (will be auto-generated if empty):'
    });
    
    // Construir mensaje de commit
    let commitMessage = `${typeResponse.type}`;
    if (scope) {
      commitMessage += `(${scope})`;
    }
    commitMessage += `: ${subjectResponse.subject}`;
    
    // Añadir descripción
    if (bodyResponse.body) {
      commitMessage += `\n\n${bodyResponse.body}`;
    }
    
    // Añadir breaking changes
    if (breakingResponse.breaking) {
      commitMessage += `\n\nBREAKING CHANGE: ${breakingResponse.breaking}`;
    }
    
    // Añadir test details
    if (testDetailsResponse.testDetails) {
      commitMessage += `\n\nTest Details: ${testDetailsResponse.testDetails}`;
    }
    
    // Añadir seguridad
    const security = securityResponse.security && securityResponse.security.trim() 
      ? securityResponse.security 
      : 'NA';
    commitMessage += `\n\nSecurity: ${security}`;
    
    // Añadir referencias
    if (ticketReference) {
      commitMessage += `\n\nRefs: ${ticketReference}`;
    }
    
    // Añadir change ID
    if (changeIdResponse.changeId) {
      commitMessage += `\n\nChange-Id: ${changeIdResponse.changeId}`;
    }
    
    // Añadir tareas de Monday
    if (mondayTasks) {
      commitMessage += `\n\nMONDAY TASKS: ${mondayTasks}`;
    }
    
    // Mostrar vista previa del mensaje
    console.log('\n--- Vista previa del mensaje de commit ---');
    console.log(commitMessage);
    console.log('----------------------------------------------');
    
    // Confirmar commit
    const confirmResponse = await prompts({
      type: 'confirm',
      name: 'value',
      message: 'Proceed with the commit?',
      initial: true
    });
    
    if (confirmResponse.value) {
      // Crear un archivo temporal con el mensaje de commit
      const tmpCommitMsg = path.join(__dirname, '..', '.git', 'COMMIT_EDITMSG');
      fs.writeFileSync(tmpCommitMsg, commitMessage, 'utf8');
      
      // Ejecutar git commit con el mensaje preparado
      try {
        console.log('Ejecutando git commit...');
        execSync('git commit -F ' + tmpCommitMsg, { stdio: 'inherit' });
        console.log('✅ Commit realizado exitosamente');
      } catch (error) {
        console.error('❌ Error al realizar el commit:', error.message);
      }
    } else {
      console.log('Commit cancelado.');
    }
    
  } catch (error) {
    console.error('Error durante el proceso de commit:', error);
  }
}

// Ejecutar la función principal
createCommit().catch(error => {
  console.error('Error inesperado:', error);
  process.exit(1);
}); 
