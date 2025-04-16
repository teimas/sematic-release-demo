#!/usr/bin/env node
require('dotenv').config();
const { execSync } = require('child_process');
const fs = require('fs');
const path = require('path');
const mondaySdk = require('monday-sdk-js');
const { GoogleGenerativeAI } = require('@google/generative-ai');

// Inicializar el SDK de Monday
const monday = mondaySdk();
monday.setToken(process.env.MONDAY_API_KEY);
monday.setApiVersion("2024-10");

// Directorio de salida para los documentos generados
const outputDir = path.join(__dirname, '..', 'release-notes');
if (!fs.existsSync(outputDir)) {
  fs.mkdirSync(outputDir, { recursive: true });
}

// Archivo de salida para el documento de Gemini
const outputFile = path.join(outputDir, `release-notes-${new Date().toISOString().split('T')[0]}.md`);
const geminiOutputFile = path.join(outputDir, `release-notes-${new Date().toISOString().split('T')[0]}_GEMINI.md`);

// Función para procesar el documento con Gemini
async function processWithGemini(document) {
  console.log('🤖 Enviando documento a Google Gemini API...');
  
  try {
    // Verificar que el token de Gemini esté configurado
    if (!process.env.GEMINI_TOKEN) {
      console.error('❌ Error: No se ha configurado GEMINI_TOKEN en el archivo .env');
      console.log('   Ejecuta "npm run config" para configurar el token de Gemini.');
      return null;
    }
    
    // Inicializar el cliente de Gemini
    const genAI = new GoogleGenerativeAI(process.env.GEMINI_TOKEN);
    // Usar gemini-1.5-pro o gemini-1.0-pro dependiendo de la disponibilidad
    const model = genAI.getGenerativeModel({ model: "gemini-1.5-pro" });
    
    console.log('⏳ Generando notas de versión con IA...');
    
    // Configuración de generación
    const generationConfig = {
      temperature: 0.7,
      topK: 40,
      topP: 0.95,
      maxOutputTokens: 8192,
    };
    
    // Llamar a la API de Gemini con la configuración adecuada
    const result = await model.generateContent({
      contents: [{ role: "user", parts: [{ text: document }] }],
      generationConfig,
    });
    
    const response = result.response.text();
    
    console.log('✅ Respuesta recibida de Google Gemini API');
    
    return response;
  } catch (error) {
    console.error('❌ Error al procesar con Gemini:', error.message);
    if (error.stack) {
      console.error('Stack:', error.stack);
    }
    
    // Intentar con un modelo alternativo
    try {
      console.log('🔄 Intentando con modelo alternativo gemini-1.0-pro...');
      const genAI = new GoogleGenerativeAI(process.env.GEMINI_TOKEN);
      const fallbackModel = genAI.getGenerativeModel({ model: "gemini-1.0-pro" });
      
      const generationConfig = {
        temperature: 0.7,
        topK: 40,
        topP: 0.95,
        maxOutputTokens: 8192,
      };
      
      const result = await fallbackModel.generateContent({
        contents: [{ role: "user", parts: [{ text: document }] }],
        generationConfig,
      });
      
      const response = result.response.text();
      console.log('✅ Respuesta recibida del modelo alternativo');
      return response;
    } catch (fallbackError) {
      console.error('❌ Error con el modelo alternativo:', fallbackError.message);
      return null;
    }
  }
}

// Función principal para generar las notas de versión
async function generateReleaseNotes() {
  console.log('🚀 Iniciando generación de notas de versión para Google Gemini...');
  
  try {
    // 1. Ejecutar semantic-release en modo dry-run para obtener la próxima versión
    console.log('⏳ Ejecutando semantic-release en modo dry-run...');
    let newVersion = 'próxima versión';
    
    try {
      const semanticReleaseOutput = execSync('npx semantic-release --dry-run', { encoding: 'utf8' });
      console.log('✅ Ejecución de semantic-release completada');
      
      // Extraer la nueva versión
      const versionMatch = semanticReleaseOutput.match(/The next release version is (\d+\.\d+\.\d+)/);
      if (versionMatch) {
        newVersion = versionMatch[1];
      }
    } catch (error) {
      console.log(`⚠️ No se pudo determinar la próxima versión con semantic-release: ${error.message}`);
      console.log('Continuando con la versión por defecto...');
    }
    
    console.log(`📝 Preparando notas para la versión ${newVersion}`);
    
    // 2. Obtener los commits relevantes desde la última versión etiquetada
    console.log('📋 Obteniendo commits desde la última versión...');
    
    // Obtener la última etiqueta
    let lastTag = '';
    try {
      lastTag = execSync('git describe --tags --abbrev=0', { encoding: 'utf8' }).trim();
      console.log(`Última etiqueta encontrada: ${lastTag}`);
    } catch (error) {
      console.log('No se encontraron etiquetas previas, utilizando todos los commits');
    }
    
    // Obtener los commits entre la última etiqueta y HEAD usando un formato que preserve la estructura
    // Usamos un delimitador especial que es poco probable que aparezca en los mensajes
    const COMMIT_DELIMITER = '---COMMIT_DELIMITER---';
    const SECTION_DELIMITER = '---SECTION_DELIMITER---';
    
    const gitLogCommand = lastTag 
      ? `git log ${lastTag}..HEAD --no-merges --pretty=format:"%H${SECTION_DELIMITER}%s${SECTION_DELIMITER}%B${SECTION_DELIMITER}%an${SECTION_DELIMITER}%ae${SECTION_DELIMITER}%ad${COMMIT_DELIMITER}"` 
      : `git log --no-merges --pretty=format:"%H${SECTION_DELIMITER}%s${SECTION_DELIMITER}%B${SECTION_DELIMITER}%an${SECTION_DELIMITER}%ae${SECTION_DELIMITER}%ad${COMMIT_DELIMITER}"`;
    
    console.log(`Ejecutando comando: ${gitLogCommand}`);
    
    // 3. Analizar cada commit y recopilar información
    const commits = [];
    const mondayTaskIds = new Set();
    
    try {
      const commitsOutput = execSync(gitLogCommand, { encoding: 'utf8' });
      // Dividir la salida en commits individuales usando el delimitador
      const commitBlocks = commitsOutput.split(COMMIT_DELIMITER).filter(block => block.trim() !== '');
      
      console.log(`📊 Se encontraron ${commitBlocks.length} commits para analizar`);
      
      if (commitBlocks.length === 0) {
        console.log('⚠️ No se encontraron commits para analizar. Verificar historial de Git.');
        console.log('Generando documento con información mínima...');
      } else {
        // Para depuración, mostrar los primeros commits
        console.log(`Muestra de commits (primeros 2):`);
        commitBlocks.slice(0, 2).forEach((block, index) => {
          console.log(`Commit ${index + 1}: ${block.substring(0, 100)}${block.length > 100 ? '...' : ''}`);
        });
        
        for (const block of commitBlocks) {
          try {
            const parts = block.split(SECTION_DELIMITER);
            
            if (parts.length < 6) {
              console.log(`Advertencia: Formato de commit inesperado, saltando: ${block.substring(0, 50)}...`);
              continue;
            }
            
            const hash = parts[0] || '';
            const subject = parts[1] || '';
            const body = parts[2] || '';
            const authorName = parts[3] || '';
            const authorEmail = parts[4] || '';
            const commitDate = parts[5] || '';
            
            // Para depuración
            console.log(`Procesando commit: ${hash.substring(0, 7)} - ${subject}`);
            
            // Extraer información del commit
            const commitInfo = {
              hash,
              subject,
              body,
              authorName,
              authorEmail,
              commitDate,
              type: extractCommitType(subject),
              scope: extractCommitScope(subject),
              description: extractCommitDescription(subject),
              breakingChanges: extractBreakingChanges(body),
              testDetails: extractTestDetails(body),
              security: extractSecurity(body),
              mondayTasks: extractMondayTasks(body),
              refs: extractRefs(body),
              changeId: extractChangeId(body)
            };
            
            // Recopilar todos los IDs de tareas de Monday
            if (commitInfo.mondayTasks && commitInfo.mondayTasks.mentions) {
              console.log(`Encontradas ${commitInfo.mondayTasks.mentions.length} tareas de Monday en el commit ${hash.substring(0, 7)}`);
              commitInfo.mondayTasks.mentions.forEach(mention => {
                if (mention.id) {
                  mondayTaskIds.add(mention.id);
                  console.log(`  - ID de tarea: ${mention.id}`);
                }
              });
            }
            
            // También buscar IDs en el scope
            if (commitInfo.scope) {
              const scopeIds = commitInfo.scope.split('|').filter(id => /^\d+$/.test(id));
              if (scopeIds.length > 0) {
                console.log(`Encontrados ${scopeIds.length} IDs de Monday en el scope del commit ${hash.substring(0, 7)}`);
                scopeIds.forEach(id => {
                  mondayTaskIds.add(id);
                  console.log(`  - ID en scope: ${id}`);
                });
              }
            }
            
            commits.push(commitInfo);
          } catch (error) {
            console.log(`Error al procesar commit: ${error.message}`);
            continue;
          }
        }
      }
      
      console.log(`📌 Procesados ${commits.length} commits y encontrados ${mondayTaskIds.size} IDs de tareas de Monday.com`);
      
    } catch (error) {
      console.error(`❌ Error al obtener commits: ${error.message}`);
      console.log('Continuando con lista de commits vacía...');
    }
    
    // 4. Obtener detalles de las tareas de Monday
    let mondayTasksDetails = [];
    if (mondayTaskIds.size > 0) {
      try {
        mondayTasksDetails = await fetchMondayTasksDetails(Array.from(mondayTaskIds));
      } catch (error) {
        console.error(`❌ Error al obtener detalles de tareas de Monday: ${error.message}`);
        console.log('Continuando sin detalles de tareas de Monday...');
      }
    } else {
      console.log('No se encontraron IDs de tareas de Monday.com en los commits');
    }
    
    // 5. Generar el documento para Google Gemini
    console.log('📄 Generando documento para Google Gemini...');
    const document = generateGeminiDocument(newVersion, commits, mondayTasksDetails);
    
    // 6. Guardar el documento en un archivo
    fs.writeFileSync(outputFile, document, 'utf8');
    console.log(`✅ Documento generado exitosamente: ${outputFile}`);
    console.log(`   Ruta completa: ${path.resolve(outputFile)}`);
    
    // 7. Procesar el documento con Gemini si está configurado
    if (process.env.GEMINI_TOKEN) {
      const geminiResponse = await processWithGemini(document);
      
      if (geminiResponse) {
        // Guardar la respuesta de Gemini en un nuevo archivo
        fs.writeFileSync(geminiOutputFile, geminiResponse, 'utf8');
        console.log(`✅ Notas de versión generadas por Gemini: ${geminiOutputFile}`);
        console.log(`   Ruta completa: ${path.resolve(geminiOutputFile)}`);
      } else {
        console.log(`❌ No se pudo generar las notas de versión con Gemini.`);
      }
    } else {
      console.log(`ℹ️ No se encontró token de Gemini en el archivo .env`);
      console.log(`   Ejecuta "npm run config" para configurar el token de Gemini y generar notas con IA.`);
    }
    
  } catch (error) {
    console.error('❌ Error al generar las notas de versión:', error);
    process.exit(1);
  }
}

// Función para extraer el tipo de commit (feat, fix, etc.)
function extractCommitType(subject) {
  if (!subject) return 'other';
  const match = subject.match(/^(\w+)(\(.*?\))?:/);
  return match ? match[1] : 'other';
}

// Función para extraer el scope del commit
function extractCommitScope(subject) {
  if (!subject) return '';
  
  // Patrón para extraer scope entre paréntesis después del tipo
  const match = subject.match(/^\w+\((.*?)\):/);
  if (!match) return '';
  
  // El scope puede contener múltiples IDs separados por | en su interior
  return match[1];
}

// Función para extraer la descripción principal del commit
function extractCommitDescription(subject) {
  if (!subject) return '';
  const match = subject.match(/^\w+(?:\(.*?\))?:\s*(.*)/);
  return match ? match[1].trim() : subject;
}

// Función para extraer breaking changes del cuerpo del commit
function extractBreakingChanges(body) {
  if (!body) return '';
  
  // Intentar diferentes patrones para mayor flexibilidad
  const patterns = [
    /BREAKING\s+CHANGE:\s*([\s\S]*?)(?:\n\n|$)/i,
    /BREAKING\s+CHANGES:\s*([\s\S]*?)(?:\n\n|$)/i,
  ];
  
  for (const pattern of patterns) {
    const match = body.match(pattern);
    if (match) {
      return match[1].trim();
    }
  }
  
  return '';
}

// Función para extraer detalles de test del cuerpo del commit
function extractTestDetails(body) {
  if (!body) return [];
  
  // Intentar diferentes patrones para mayor flexibilidad
  const patterns = [
    /Test\s+Details:\s*([\s\S]*?)(?:\n\n|$)/i,
    /Test[s]?:\s*([\s\S]*?)(?:\n\n|$)/i,
    /testDetails\s*([\s\S]*?)(?:\n\n|$)/i
  ];
  
  for (const pattern of patterns) {
    const match = body.match(pattern);
    if (match) {
      // Dividir por líneas y filtrar las que empiezan con -
      const lines = match[1].trim().split('\n')
        .map(line => line.trim());
      
      // Si hay líneas que empiezan con -, filtramos por ellas
      const bulletLines = lines.filter(line => line.startsWith('-'));
      
      if (bulletLines.length > 0) {
        return bulletLines.map(line => line.substring(1).trim());
      }
      
      // Si no hay líneas con -, devolvemos todas las líneas
      return lines;
    }
  }
  
  return [];
}

// Función para extraer información de seguridad
function extractSecurity(body) {
  if (!body) return 'NA';
  
  // Intentar diferentes patrones para mayor flexibilidad
  const patterns = [
    /Security:\s*([\s\S]*?)(?:\n\n|$)/i,
    /security\s*([\s\S]*?)(?:\n\n|$)/i
  ];
  
  for (const pattern of patterns) {
    const match = body.match(pattern);
    if (match) {
      return match[1].trim() || 'NA';
    }
  }
  
  return 'NA';
}

// Función para extraer referencias a tickets
function extractRefs(body) {
  if (!body) return '';
  
  // Intentar diferentes patrones para mayor flexibilidad
  const patterns = [
    /Refs:\s*([\s\S]*?)(?:\n\n|$)/i,
    /references\s*([\s\S]*?)(?:\n\n|$)/i
  ];
  
  for (const pattern of patterns) {
    const match = body.match(pattern);
    if (match) {
      return match[1].trim();
    }
  }
  
  return '';
}

// Función para extraer Change-Id
function extractChangeId(body) {
  if (!body) return '';
  
  // Intentar diferentes patrones para mayor flexibilidad
  const patterns = [
    /Change-Id:\s*([\s\S]*?)(?:\n\n|$)/i,
    /Change-ID:\s*([\s\S]*?)(?:\n\n|$)/i,
    /changeId\s*([\s\S]*?)(?:\n\n|$)/i
  ];
  
  for (const pattern of patterns) {
    const match = body.match(pattern);
    if (match) {
      return match[1].trim();
    }
  }
  
  return '';
}

// Función para extraer tareas de Monday del cuerpo del commit
function extractMondayTasks(body) {
  if (!body) return null;
  
  // Primero intentar extraer usando el patrón "MONDAY TASKS:"
  const mondayTasksMatch = body.match(/MONDAY\s+TASKS:\s*([\s\S]*?)(?:\n\n|$)/i);
  
  if (!mondayTasksMatch) {
    return null;
  }
  
  const tasksText = mondayTasksMatch[1].trim();
  console.log(`Encontrado texto de tareas de Monday: ${tasksText.substring(0, 100)}${tasksText.length > 100 ? '...' : ''}`);
  
  // Extraer las menciones de tareas
  const mentions = [];
  const taskLines = tasksText.split('\n');
  
  for (const line of taskLines) {
    // Buscar patrones como "[PROJECT] Title (ID: 123456789, URL: url)"
    const cleanLine = line.replace(/^-\s*/, '').trim();
    const idMatch = cleanLine.match(/ID:\s*(\d+)/i);
    const urlMatch = cleanLine.match(/URL:\s*([^,\)]+)/i);
    const titleMatch = cleanLine.match(/^(.*?)\s*\(ID:/);
    
    if (idMatch) {
      mentions.push({
        id: idMatch[1],
        title: titleMatch ? titleMatch[1].trim() : '',
        url: urlMatch ? urlMatch[1].trim() : ''
      });
      console.log(`  - Tarea encontrada: ID=${idMatch[1]}, Título=${titleMatch ? titleMatch[1].trim() : 'Sin título'}`);
    }
  }
  
  return {
    raw: tasksText,
    mentions
  };
}

// Función para obtener detalles de las tareas de Monday
async function fetchMondayTasksDetails(taskIds) {
  if (!taskIds.length) return [];
  
  console.log(`📦 Consultando detalles de ${taskIds.length} tareas en Monday.com...`);
  
  try {
    // Verificar que el token de API esté configurado
    if (!process.env.MONDAY_API_KEY) {
      console.error('❌ Error: No se ha configurado MONDAY_API_KEY en el archivo .env');
      return [];
    }
    
    // Asegurar que monday esté correctamente inicializado
    monday.setToken(process.env.MONDAY_API_KEY);
    monday.setApiVersion("2024-10");
    
    // Dividir las consultas en bloques más pequeños para evitar límites de la API
    const BATCH_SIZE = 10;
    let allItems = [];
    
    // Procesar los IDs en lotes
    for (let i = 0; i < taskIds.length; i += BATCH_SIZE) {
      const batchIds = taskIds.slice(i, i + BATCH_SIZE);
      console.log(`Procesando lote ${Math.floor(i/BATCH_SIZE) + 1} con ${batchIds.length} tareas...`);
      
      // Consultar detalles de las tareas en Monday.com
      const query = `query ($itemIds: [ID!]) {
        items (ids: $itemIds) {
          id
          name
          state
          board {
            id
            name
          }
          group {
            id
            title
          }
          column_values {
            id
            type
            text
            value
          }
          updates(limit: 5) {
            id
            body
            created_at
            creator {
              id
              name
            }
          }
        }
      }`;
      
      const variables = { itemIds: batchIds };
      
      console.log(`Consultando datos para IDs: ${batchIds.join(', ')}`);
      
      try {
        const response = await monday.api(query, { variables });
        
        if (response.errors) {
          console.error('⚠️ Errores en la respuesta de Monday:', response.errors);
          continue;
        }
        
        if (response.data && response.data.items) {
          console.log(`✅ Obtenidos datos de ${response.data.items.length} tareas`);
          allItems = [...allItems, ...response.data.items];
        } else {
          console.log('⚠️ No se encontraron items en la respuesta de este lote');
        }
      } catch (batchError) {
        console.error(`❌ Error en lote ${Math.floor(i/BATCH_SIZE) + 1}:`, batchError.message);
        continue;
      }
      
      // Esperar entre lotes para no exceder límites de rate
      if (i + BATCH_SIZE < taskIds.length) {
        console.log('Esperando 1 segundo antes del siguiente lote...');
        await new Promise(resolve => setTimeout(resolve, 1000));
      }
    }
    
    console.log(`✅ Se obtuvieron detalles de ${allItems.length} tareas de Monday.com de ${taskIds.length} solicitadas`);
    return allItems;
  } catch (error) {
    console.error('❌ Error al consultar tareas en Monday.com:', error.message);
    if (error.data) {
      console.error('Detalles del error:', JSON.stringify(error.data, null, 2));
    }
    return [];
  }
}

// Función para generar el documento estructurado para Google Gemini
function generateGeminiDocument(version, commits, mondayTasks) {
  // Crear un mapeo de ID de tarea a detalles para búsqueda rápida
  const taskDetailsMap = mondayTasks.reduce((map, task) => {
    map[task.id] = task;
    return map;
  }, {});
  
  // Agrupar commits por tipo
  const commitsByType = groupCommitsByType(commits);
  
  // Generar el documento
  let document = '';
  
  // Encabezado
  document += `# Datos para Generación de Notas de Versión ${version}\n\n`;
  document += `## Información General\n\n`;
  document += `- **Versión**: ${version}\n`;
  document += `- **Fecha**: ${new Date().toLocaleDateString('es-ES', { day: '2-digit', month: '2-digit', year: 'numeric' })}\n`;
  document += `- **Total de Commits**: ${commits.length}\n`;
  document += `- **Tareas de Monday relacionadas**: ${mondayTasks.length}\n\n`;
  
  // Instrucciones para Gemini
  document += `## Instrucciones\n\n`;
  document += `Necesito que generes unas notas de versión detalladas en español, basadas en los datos proporcionados a continuación. `;
  document += `Estas notas deben estar dirigidas a usuarios finales y equipos técnicos, destacando las nuevas funcionalidades, correcciones y mejoras. `;
  
  // Resumen de cambios por tipo
  document += `## Resumen de Cambios\n\n`;
  
  // Nuevas características (feat)
  if (commitsByType.feat && commitsByType.feat.length > 0) {
    document += `### Nuevas Funcionalidades (${commitsByType.feat.length})\n\n`;
    commitsByType.feat.forEach(commit => {
      document += `- **${commit.description}** [${commit.hash.substring(0, 7)}] - ${commit.authorName} <${commit.authorEmail}> (${commit.commitDate})\n`;
      if (commit.body) {
        document += `  - Detalles: ${formatMultilineText(commit.body)}\n`;
      }
    });
    document += `\n`;
  }
  
  // Correcciones (fix)
  if (commitsByType.fix && commitsByType.fix.length > 0) {
    document += `### Correcciones (${commitsByType.fix.length})\n\n`;
    commitsByType.fix.forEach(commit => {
      document += `- **${commit.description}** [${commit.hash.substring(0, 7)}] - ${commit.authorName} <${commit.authorEmail}> (${commit.commitDate})\n`;
      if (commit.body) {
        document += `  - Detalles: ${formatMultilineText(commit.body)}\n`;
      }
    });
    document += `\n`;
  }
  
  // Otros tipos de commits
  const otherTypes = Object.keys(commitsByType).filter(type => !['feat', 'fix'].includes(type));
  if (otherTypes.length > 0) {
    otherTypes.forEach(type => {
      if (commitsByType[type] && commitsByType[type].length > 0) {
        document += `### ${getTypeTitle(type)} (${commitsByType[type].length})\n\n`;
        commitsByType[type].forEach(commit => {
          document += `- **${commit.description}** [${commit.hash.substring(0, 7)}] - ${commit.authorName} <${commit.authorEmail}> (${commit.commitDate})\n`;
          if (commit.body) {
            document += `  - Detalles: ${formatMultilineText(commit.body)}\n`;
          }
        });
        document += `\n`;
      }
    });
  }
  
  // Cambios que rompen compatibilidad
  const breakingChanges = commits.filter(commit => commit.breakingChanges);
  if (breakingChanges.length > 0) {
    document += `## Cambios que Rompen Compatibilidad\n\n`;
    breakingChanges.forEach(commit => {
      document += `- **${commit.description}** [${commit.hash.substring(0, 7)}] - ${commit.authorName} <${commit.authorEmail}> (${commit.commitDate})\n`;
      document += `  - Detalles: ${formatMultilineText(commit.breakingChanges)}\n`;
    });
    document += `\n`;
  }
  
  // Detalles de tareas de Monday
  if (mondayTasks.length > 0) {
    document += `## Detalles de Tareas de Monday\n\n`;
    
    mondayTasks.forEach(task => {
      document += `### ${task.name} (ID: ${task.id})\n\n`;
      document += `- **Estado**: ${task.state}\n`;
      document += `- **Tablero**: ${task.board?.name || 'N/A'} (ID: ${task.board?.id || 'N/A'})\n`;
      document += `- **Grupo**: ${task.group?.title || 'N/A'}\n`;
      
      // Información de columnas relevantes
      if (task.column_values && task.column_values.length > 0) {
        document += `- **Detalles**:\n`;
        
        // Filtrar columnas con valores
        const relevantColumns = task.column_values.filter(col => col.text && col.text.trim() !== '');
        
        if (relevantColumns.length > 0) {
          relevantColumns.forEach(col => {
            document += `  - ${col.id}: ${col.text}\n`;
          });
        } else {
          document += `  - No hay detalles adicionales disponibles\n`;
        }
      }
      
      // Actualizaciones recientes
      if (task.updates && task.updates.length > 0) {
        document += `- **Actualizaciones Recientes**:\n`;
        
        // Mostrar las 3 actualizaciones más recientes
        task.updates.slice(0, 3).forEach(update => {
          const date = new Date(update.created_at).toLocaleDateString('es-ES', {
            day: '2-digit',
            month: '2-digit',
            year: 'numeric'
          });
          
          document += `  - ${date} por ${update.creator?.name || 'Usuario'}: ${update.body.substring(0, 100)}${update.body.length > 100 ? '...' : ''}\n`;
        });
      }
      
      // Commits relacionados
      const relatedCommits = commits.filter(commit => {
        // Buscar en scope
        if (commit.scope && commit.scope.split('|').includes(task.id)) {
          return true;
        }
        
        // Buscar en mondayTasks
        if (commit.mondayTasks && commit.mondayTasks.mentions) {
          return commit.mondayTasks.mentions.some(mention => mention.id === task.id);
        }
        
        return false;
      });
      
      if (relatedCommits.length > 0) {
        document += `- **Commits Relacionados**:\n`;
        relatedCommits.forEach(commit => {
          document += `  - ${commit.type}: ${commit.description} [${commit.hash.substring(0, 7)}]\n`;
        });
      }
      
      document += `\n`;
    });
  }
  
  // Detalles completos de commits
  document += `## Detalles Completos de Commits\n\n`;
  
  commits.forEach(commit => {
    document += `### ${commit.type}${commit.scope ? `(${commit.scope})` : ''}: ${commit.description} [${commit.hash.substring(0, 7)}]\n\n`;
    
    document += `**Autor**: ${commit.authorName} <${commit.authorEmail}>\n`;
    document += `**Fecha**: ${commit.commitDate}\n\n`;
    
    if (commit.body) {
      document += `${formatMultilineText(commit.body)}\n\n`;
    }
    
    if (commit.testDetails && commit.testDetails.length > 0) {
      document += `**Pruebas**:\n`;
      commit.testDetails.forEach(test => {
        document += `- ${test}\n`;
      });
      document += `\n`;
    }
    
    if (commit.security && commit.security !== 'NA') {
      document += `**Seguridad**: ${commit.security}\n\n`;
    }
    
    if (commit.mondayTasks && commit.mondayTasks.mentions && commit.mondayTasks.mentions.length > 0) {
      document += `**Tareas relacionadas**:\n`;
      
      commit.mondayTasks.mentions.forEach(mention => {
        const taskDetails = taskDetailsMap[mention.id];
        const taskName = taskDetails ? taskDetails.name : mention.title;
        const taskState = taskDetails ? taskDetails.state : 'Desconocido';
        
        document += `- ${taskName} (ID: ${mention.id}, Estado: ${taskState})\n`;
      });
      
      document += `\n`;
    }
    
    document += `---\n\n`;
  });

  document += `La plantilla a utilizar para generar el documento tiene que ser la siguiente. Fijate en todo lo que hay y emúlalo por completo.`;
  // Leer el contenido de la plantilla y añadirlo al documento
  try {
    const plantillaPath = path.join(__dirname, 'plantilla.md');
    if (fs.existsSync(plantillaPath)) {
      const plantillaContent = fs.readFileSync(plantillaPath, 'utf8');
      document += `\n\n${plantillaContent}`;
      console.log(`✅ Plantilla cargada exitosamente: ${plantillaPath}`);
    } else {
      console.log(`⚠️ No se encontró el archivo de plantilla: ${plantillaPath}`);
    }
  } catch (error) {
    console.error(`❌ Error al leer la plantilla: ${error.message}`);
  }

  return document;
}

// Función auxiliar para agrupar commits por tipo
function groupCommitsByType(commits) {
  return commits.reduce((groups, commit) => {
    const type = commit.type;
    if (!groups[type]) {
      groups[type] = [];
    }
    groups[type].push(commit);
    return groups;
  }, {});
}

// Función auxiliar para formatear texto multilinea
function formatMultilineText(text) {
  if (!text) return '';
  return text.split('\n')
    .map(line => line.trim())
    .filter(line => line)
    .join(' | ');
}

// Función auxiliar para obtener título legible de un tipo de commit
function getTypeTitle(type) {
  const titles = {
    feat: 'Nuevas Funcionalidades',
    fix: 'Correcciones',
    docs: 'Documentación',
    style: 'Estilo',
    refactor: 'Refactorizaciones',
    perf: 'Mejoras de Rendimiento',
    test: 'Pruebas',
    build: 'Construcción',
    ci: 'Integración Continua',
    chore: 'Tareas',
    revert: 'Reversiones',
    other: 'Otros Cambios'
  };
  
  return titles[type] || `${type.charAt(0).toUpperCase() + type.slice(1)}`;
}

// Ejecutar la función principal si este archivo se ejecuta directamente
if (require.main === module) {
  generateReleaseNotes()
    .then(() => {
      console.log('🎉 Proceso completado con éxito!');
    })
    .catch(err => {
      console.error('💥 Error durante la ejecución:', err);
      process.exit(1);
    });
}

// Exportar la función principal para permitir su uso como módulo
module.exports = {
  generateReleaseNotes
}; 
