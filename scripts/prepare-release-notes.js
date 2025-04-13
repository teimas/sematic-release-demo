#!/usr/bin/env node
require('dotenv').config();
const { execSync } = require('child_process');
const fs = require('fs');
const path = require('path');
const mondaySdk = require('monday-sdk-js');

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
    
    // Obtener los commits entre la última etiqueta y HEAD
    const gitLogCommand = lastTag 
      ? `git log ${lastTag}..HEAD --pretty=format:"%H|%s|%b" --no-merges` 
      : 'git log --pretty=format:"%H|%s|%b" --no-merges';
    
    console.log(`Ejecutando comando: ${gitLogCommand}`);
    
    // 3. Analizar cada commit y recopilar información
    const commits = [];
    const mondayTaskIds = new Set();
    
    try {
      const commitsOutput = execSync(gitLogCommand, { encoding: 'utf8' });
      const commitLines = commitsOutput.split('\n').filter(line => line.trim() !== '');
      
      console.log(`📊 Se encontraron ${commitLines.length} commits para analizar`);
      
      if (commitLines.length === 0) {
        console.log('⚠️ No se encontraron commits para analizar. Verificar historial de Git.');
        console.log('Generando documento con información mínima...');
      } else {
        // Para depuración, mostrar los primeros commits
        console.log(`Muestra de commits (primeros 2):`);
        commitLines.slice(0, 2).forEach((line, index) => {
          console.log(`Commit ${index + 1}: ${line.substring(0, 100)}${line.length > 100 ? '...' : ''}`);
        });
        
        for (const line of commitLines) {
          try {
            const parts = line.split('|');
            
            if (parts.length < 2) {
              console.log(`Advertencia: Formato de commit inesperado, saltando: ${line.substring(0, 50)}...`);
              continue;
            }
            
            const hash = parts[0] || '';
            const subject = parts[1] || '';
            const bodyParts = parts.slice(2);
            const body = bodyParts.join('|');
            
            // Extraer información del commit
            const commitInfo = {
              hash,
              subject,
              body,
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
              commitInfo.mondayTasks.mentions.forEach(mention => {
                if (mention.id) {
                  mondayTaskIds.add(mention.id);
                }
              });
            }
            
            // También buscar IDs en el scope
            if (commitInfo.scope) {
              const scopeIds = commitInfo.scope.split('|').filter(id => /^\d+$/.test(id));
              scopeIds.forEach(id => mondayTaskIds.add(id));
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
  const match = subject.match(/^\w+\((.*?)\):/);
  return match ? match[1] : '';
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
  const match = body.match(/BREAKING CHANGE:([\s\S]*?)(?:\n\n|$)/);
  return match ? match[1].trim() : '';
}

// Función para extraer detalles de test del cuerpo del commit
function extractTestDetails(body) {
  if (!body) return [];
  const match = body.match(/Test Details:([\s\S]*?)(?:\n\n|$)/);
  if (!match) return [];

  // Dividir por líneas y filtrar las que empiezan con -
  return match[1].trim().split('\n')
    .map(line => line.trim())
    .filter(line => line.startsWith('-'))
    .map(line => line.substring(1).trim());
}

// Función para extraer información de seguridad
function extractSecurity(body) {
  if (!body) return 'NA';
  const match = body.match(/Security:([\s\S]*?)(?:\n\n|$)/);
  return match ? match[1].trim() : 'NA';
}

// Función para extraer referencias a tickets
function extractRefs(body) {
  if (!body) return '';
  const match = body.match(/Refs:([\s\S]*?)(?:\n\n|$)/);
  return match ? match[1].trim() : '';
}

// Función para extraer Change-Id
function extractChangeId(body) {
  if (!body) return '';
  const match = body.match(/Change-Id:([\s\S]*?)(?:\n\n|$)/);
  return match ? match[1].trim() : '';
}

// Función para extraer tareas de Monday del cuerpo del commit
function extractMondayTasks(body) {
  if (!body) return null;
  const match = body.match(/MONDAY TASKS:([\s\S]*?)(?:\n\n|$)/);
  if (!match) return null;
  
  const tasksText = match[1].trim();
  
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
          title
          text
          value
          type
        }
        updates {
          id
          body
          created_at
          creator {
            name
          }
        }
      }
    }`;
    
    const variables = { itemIds: taskIds };
    const response = await monday.api(query, { variables });
    
    if (response.data && response.data.items) {
      console.log(`✅ Se obtuvieron detalles de ${response.data.items.length} tareas de Monday.com`);
      return response.data.items;
    } else {
      console.log('❌ No se pudieron obtener detalles de Monday.com', response.errors);
      return [];
    }
  } catch (error) {
    console.error('❌ Error al consultar tareas en Monday.com:', error.message);
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
  document += `Organiza la información por categorías (Nuevas funcionalidades, Correcciones, Mejoras, etc.) y destaca las tareas más importantes. `;
  document += `Incluye menciones a las tareas de Monday.com relevantes y sus detalles cuando sea apropiado. `;
  document += `El tono debe ser profesional pero accesible, evitando jerga excesivamente técnica. `;
  document += `La estructura debe ser clara con encabezados, viñetas y párrafos concisos.\n\n`;
  
  // Resumen de cambios por tipo
  document += `## Resumen de Cambios\n\n`;
  
  // Nuevas características (feat)
  if (commitsByType.feat && commitsByType.feat.length > 0) {
    document += `### Nuevas Funcionalidades (${commitsByType.feat.length})\n\n`;
    commitsByType.feat.forEach(commit => {
      document += `- **${commit.description}** [${commit.hash.substring(0, 7)}]\n`;
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
      document += `- **${commit.description}** [${commit.hash.substring(0, 7)}]\n`;
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
          document += `- **${commit.description}** [${commit.hash.substring(0, 7)}]\n`;
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
      document += `- **${commit.description}** [${commit.hash.substring(0, 7)}]\n`;
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
            document += `  - ${col.title || col.id}: ${col.text}\n`;
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
