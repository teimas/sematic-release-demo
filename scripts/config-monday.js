#!/usr/bin/env node
const prompts = require('prompts');
const fs = require('fs');
const path = require('path');
const monday = require('monday-sdk-js')();

// Establece la versión de la API según la documentación
monday.setApiVersion("2024-10");

// Asegura que el directorio scripts existe
const scriptsDir = path.join(__dirname);
if (!fs.existsSync(scriptsDir)) {
  fs.mkdirSync(scriptsDir, { recursive: true });
}

async function configureMonday() {
  console.log('📚 Configuración de acceso a la API de Monday.com');
  console.log('------------------------------------------------');
  
  const envPath = path.join(process.cwd(), '.env');
  const existingEnv = fs.existsSync(envPath) ? fs.readFileSync(envPath, 'utf8') : '';
  
  // Extraer valores existentes si los hay
  const existingApiKey = (existingEnv.match(/MONDAY_API_KEY=(.+)/) || [])[1] || '';
  
  const questions = [
    {
      type: 'password',
      name: 'apiKey',
      message: 'Ingresa tu token de API de Monday.com:',
      initial: existingApiKey,
      validate: value => value.length > 0 ? true : 'El token de API es obligatorio'
    },
    {
      type: 'text',
      name: 'accountSlug',
      message: 'Slug de la cuenta (subdominio de Monday.com):',
      initial: (existingEnv.match(/ACCOUNT_SLUG=(.+)/) || [])[1] || '',
      validate: value => value.length > 0 ? true : 'El slug de la cuenta es obligatorio'
    },
    {
      type: 'text',
      name: 'boardId',
      message: 'ID del tablero principal (opcional):',
      initial: (existingEnv.match(/MONDAY_BOARD_ID=(.+)/) || [])[1] || ''
    }
  ];

  try {
    const response = await prompts(questions);
    
    // Verificar que la API key es válida
    if (response.apiKey) {
      try {
        monday.setToken(response.apiKey);
        
        // Intenta hacer una llamada simple para verificar la autenticación
        console.log('Verificando acceso a la API...');
        const meQuery = 'query { me { name email } }';
        const result = await monday.api(meQuery);
        
        if (result.data && result.data.me) {
          console.log(`✅ Conexión exitosa. Bienvenido, ${result.data.me.name} (${result.data.me.email})!`);
        } else {
          console.log('⚠️ La verificación fue inconclusa, pero se guardará la configuración de todos modos.');
        }
      } catch (error) {
        console.error('❌ Error al verificar la API key:', error.message);
        const continueAnyway = await prompts({
          type: 'confirm',
          name: 'value',
          message: '¿Deseas guardar esta configuración de todos modos?',
          initial: false
        });
        
        if (!continueAnyway.value) {
          console.log('Configuración cancelada. No se guardaron cambios.');
          return;
        }
      }
    }
    
    // Guarda la configuración en .env
    let newEnv = existingEnv;
    
    // Actualizar o añadir MONDAY_API_KEY
    if (response.apiKey) {
      if (newEnv.includes('MONDAY_API_KEY=')) {
        newEnv = newEnv.replace(/MONDAY_API_KEY=.+/, `MONDAY_API_KEY=${response.apiKey}`);
      } else {
        newEnv += `\nMONDAY_API_KEY=${response.apiKey}`;
      }
    }
    
    // Actualizar o añadir ACCOUNT_SLUG
    if (response.accountSlug) {
      if (newEnv.includes('ACCOUNT_SLUG=')) {
        newEnv = newEnv.replace(/ACCOUNT_SLUG=.+/, `ACCOUNT_SLUG=${response.accountSlug}`);
      } else {
        newEnv += `\nACCOUNT_SLUG=${response.accountSlug}`;
      }
      
      // Añadir o actualizar URL template
      const mondayUrlTemplate = `https://${response.accountSlug}.monday.com/boards/{board_id}/pulses/{item_id}`;
      if (newEnv.includes('MONDAY_URL_TEMPLATE=')) {
        newEnv = newEnv.replace(/MONDAY_URL_TEMPLATE=.+/, `MONDAY_URL_TEMPLATE=${mondayUrlTemplate}`);
      } else {
        newEnv += `\nMONDAY_URL_TEMPLATE=${mondayUrlTemplate}`;
      }
    }
    
    // Actualizar o añadir MONDAY_BOARD_ID
    if (response.boardId) {
      if (newEnv.includes('MONDAY_BOARD_ID=')) {
        newEnv = newEnv.replace(/MONDAY_BOARD_ID=.+/, `MONDAY_BOARD_ID=${response.boardId}`);
      } else {
        newEnv += `\nMONDAY_BOARD_ID=${response.boardId}`;
      }
    }
    
    // Asegurarse de que empiece con una nueva línea si ya había contenido
    if (existingEnv && !newEnv.startsWith('\n')) {
      newEnv = '\n' + newEnv;
    }
    
    // Eliminar líneas vacías duplicadas
    newEnv = newEnv.replace(/\n\s*\n/g, '\n');
    
    // Guardar el archivo .env
    fs.writeFileSync(envPath, newEnv.trim());
    
    console.log('✅ Configuración guardada exitosamente en .env');
    console.log('');
    console.log('Puedes usar el SDK de Monday en tus scripts con:');
    console.log('```');
    console.log('const mondaySdk = require("monday-sdk-js");');
    console.log('const monday = mondaySdk();');
    console.log('monday.setToken(process.env.MONDAY_API_KEY);');
    console.log('monday.setApiVersion("2024-10");');
    console.log('```');
    
    console.log('');
    console.log('URL de Monday.com configurada:');
    console.log(`https://${response.accountSlug}.monday.com/boards/{board_id}/pulses/{item_id}`);
    console.log('');
    console.log('Puedes generar URLs para tareas con:');
    console.log('```');
    console.log('const generateMondayUrl = (boardId, itemId) => {');
    console.log('  return process.env.MONDAY_URL_TEMPLATE');
    console.log('    .replace("{board_id}", boardId)');
    console.log('    .replace("{item_id}", itemId);');
    console.log('};');
    console.log('```');
    
    // Verificar si el script search-task.js ya existe
    const examplePath = path.join(process.cwd(), 'scripts', 'search-task.js');
    const scriptExists = fs.existsSync(examplePath);
    
    const updateScript = scriptExists ? await prompts({
      type: 'confirm',
      name: 'value',
      message: 'El script de búsqueda ya existe. ¿Deseas actualizarlo?',
      initial: false
    }) : { value: true };
    
    if (updateScript.value) {
      // Crear un archivo de ejemplo para buscar tareas por nombre
      const exampleContent = `#!/usr/bin/env node
require('dotenv').config();
const mondaySdk = require('monday-sdk-js');
const monday = mondaySdk();
const prompts = require('prompts');

// Configura el token de API desde las variables de entorno
monday.setToken(process.env.MONDAY_API_KEY);
// Establece la versión de la API según la documentación (última versión disponible)
monday.setApiVersion("2024-10");

// Función para generar URL de Monday
const generateMondayUrl = (boardId, itemId) => {
  const template = process.env.MONDAY_URL_TEMPLATE || 
                  \`\https://${process.env.ACCOUNT_SLUG}.monday.com/boards/{board_id}/pulses/{item_id}\`;
  return template
    .replace('{board_id}', boardId)
    .replace('{item_id}', itemId);
};

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
      const boardsQuery = \`query { 
        boards (limit: 10) { 
          id 
          name 
          description 
          state 
        } 
      }\`;
      
      const result = await monday.api(boardsQuery);
      
      if (result.data && result.data.boards) {
        console.log('Tableros encontrados:');
        const activeBoards = result.data.boards.filter(board => board.state === "active");
        
        if (activeBoards.length === 0) {
          console.log('No se encontraron tableros activos.');
        } else {
          activeBoards.forEach(board => {
            console.log(\`- \${board.name} (ID: \${board.id})\`);
            if (board.description) {
              console.log(\`  Descripción: \${board.description}\`);
            }
          });
        }
        
        console.log('\\nPara mejorar la búsqueda, configura un ID de tablero específico ejecutando "npm run config"');
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
      searchQuery = \`query ($boardId: [ID!], $limit: Int!, $queryParams: ItemsQuery) {
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
      }\`;
      variables.boardId = [boardId]; // Ya no necesitamos convertirlo a Int
    } else {
      // Búsqueda global usando el endpoint tradicional
      searchQuery = \`query ($limit: Int!, $queryParams: ItemsQuery) {
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
      }\`;
    }
    
    console.log('Realizando búsqueda...');
    const result = await monday.api(searchQuery, { variables });
    
    // Procesamiento según la estructura de respuesta
    if (boardId && result.data && result.data.boards && result.data.boards.length > 0) {
      // Caso de búsqueda en un tablero específico
      const board = result.data.boards[0];
      console.log(\`\\nBuscando en tablero: \${board.name}\`);
      
      if (board.items_page && board.items_page.items) {
        // Filtrado adicional del lado del cliente si se especificó un estado
        let items = board.items_page.items.filter(item => item.state === "active");
        
        if (statusFilter) {
          items = items.filter(item => {
            // Buscamos una columna de estado que contenga el texto especificado
            const statusColumn = item.column_values.find(col => 
              col.type === "color" || col.type === "status" || col.id === "status" || 
              (col.text && col.text.toLowerCase().includes(statusFilter.toLowerCase()))
            );
            
            return statusColumn && statusColumn.text && 
                   statusColumn.text.toLowerCase().includes(statusFilter.toLowerCase());
          });
        }
        
        if (items.length === 0) {
          console.log('No se encontraron tareas activas que coincidan con los criterios de búsqueda.');
        } else {
          console.log(\`\\nTareas encontradas (\${items.length}):\`);
          items.forEach(item => {
            console.log(\`- \${item.name} (ID: \${item.id})\`);
            console.log(\`  Actualizado: \${new Date(item.updated_at).toLocaleString()}\`);
            console.log(\`  Creado por: \${item.creator?.name || 'N/A'}\`);
            console.log(\`  Grupo: \${item.group?.title || 'N/A'}\`);
            console.log(\`  URL: \${generateMondayUrl(boardId, item.id)}\`);
            
            console.log('  Detalles:');
            if (item.column_values && item.column_values.length > 0) {
              const visibleColumns = item.column_values.filter(col => col.text && col.text.trim() !== '');
              if (visibleColumns.length > 0) {
                visibleColumns.forEach(col => {
                  console.log(\`    \${col.type || col.id}: \${col.text}\`);
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
      let items = result.data.items_page.items.filter(item => item.state === "active");
      
      if (items.length === 0) {
        console.log('No se encontraron tareas activas que coincidan con los criterios de búsqueda.');
      } else {
        console.log(\`\\nTareas encontradas (\${items.length}):\`);
        items.forEach(item => {
          console.log(\`- \${item.name} (ID: \${item.id})\`);
          console.log(\`  Actualizado: \${new Date(item.updated_at).toLocaleString()}\`);
          console.log(\`  Creado por: \${item.creator?.name || 'N/A'}\`);
          console.log(\`  Tablero: \${item.board?.name} (ID: \${item.board?.id})\`);
          console.log(\`  Grupo: \${item.group?.title || 'N/A'}\`);
          console.log(\`  URL: \${generateMondayUrl(item.board?.id, item.id)}\`);
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
      console.log('\\n⚠️ Parece que hay un problema con la autenticación.');
      console.log('Ejecuta "npm run config" para actualizar tu token de API de Monday.com.');
    }
  }
}

searchTasks().catch(error => {
  console.error('Error inesperado:', error);
});`;

      // Asegurar que existe el directorio de scripts
      if (!fs.existsSync(path.join(process.cwd(), 'scripts'))) {
        fs.mkdirSync(path.join(process.cwd(), 'scripts'), { recursive: true });
      }

      fs.writeFileSync(examplePath, exampleContent.trim());
      // Hacer el archivo ejecutable en sistemas Unix
      try {
        fs.chmodSync(examplePath, '755');
      } catch (error) {
        // Ignorar en Windows
      }
      
      console.log('');
      console.log(`✅ Se ha ${scriptExists ? 'actualizado' : 'creado'} el script de búsqueda:`);
      console.log('   npm run search-task');
    }
    
  } catch (error) {
    console.error('Error durante la configuración:', error);
  }
}

configureMonday().catch(console.error); 