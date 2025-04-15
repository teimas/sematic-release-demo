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
  const existingGeminiToken = (existingEnv.match(/GEMINI_TOKEN=(.+)/) || [])[1] || '';
  
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
    },
    {
      type: 'password',
      name: 'geminiToken',
      message: 'Ingresa tu token de API de Google Gemini (opcional):',
      initial: existingGeminiToken
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
    
    // Actualizar o añadir GEMINI_TOKEN
    if (response.geminiToken) {
      if (newEnv.includes('GEMINI_TOKEN=')) {
        newEnv = newEnv.replace(/GEMINI_TOKEN=.+/, `GEMINI_TOKEN=${response.geminiToken}`);
      } else {
        newEnv += `\nGEMINI_TOKEN=${response.geminiToken}`;
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
    
    // Mostrar información sobre Gemini API si se configuró
    if (response.geminiToken) {
      console.log('');
      console.log('🤖 Google Gemini API configurada');
      console.log('--------------------------------');
      console.log('Puedes usar la API de Google Gemini en tus scripts con:');
      console.log('```');
      console.log('const { GoogleGenerativeAI } = require("@google/generative-ai");');
      console.log('const genAI = new GoogleGenerativeAI(process.env.GEMINI_TOKEN);');
      console.log('const model = genAI.getGenerativeModel({ model: "gemini-pro" });');
      console.log('');
      console.log('// Ejemplo de uso:');
      console.log('async function generateReleaseNotes(prompt) {');
      console.log('  const result = await model.generateContent(prompt);');
      console.log('  return result.response.text();');
      console.log('}');
      console.log('```');
      console.log('');
      console.log('Para usar el script de generación de notas de versión:');
      console.log('```');
      console.log('npm run release-notes');
      console.log('```');
    }
    
  } catch (error) {
    console.error('Error durante la configuración:', error);
  }
}

configureMonday().catch(console.error); 