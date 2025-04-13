#!/usr/bin/env node
const { spawn } = require('child_process');
const readline = require('readline');

// Create readline interface for user input
const rl = readline.createInterface({
  input: process.stdin,
  output: process.stdout
});

// Function to execute git-cz
function runCommitizen() {
  console.log('Starting commitizen...');
  
  // Spawn git-cz process
  const gitCz = spawn('npx', ['git-cz'], { 
    stdio: 'inherit',
    shell: true 
  });

  // Handle process completion
  gitCz.on('close', (code) => {
    process.exit(code);
  });
}

// Function to check for Monday tasks when footer is requested
function attachMondayTasksToCommit() {
  rl.question('¿Deseas buscar tareas de Monday para incluir en el commit? (s/n): ', (answer) => {
    rl.close();
    
    if (answer.toLowerCase() === 's' || answer.toLowerCase() === 'si' || answer.toLowerCase() === 'sí') {
      // Run Monday task selector with inherit stdio for interactive prompts
      console.log('Ejecutando buscador de tareas de Monday...');
      
      const selectorProcess = spawn('npm', ['run', 'monday-selector'], { 
        stdio: 'inherit',
        shell: true 
      });

      selectorProcess.on('close', (code) => {
        if (code !== 0) {
          console.error(`El selector de tareas salió con código ${code}`);
        }
        
        // Run commitizen after the selector completes
        runCommitizen();
      });
    } else {
      // Skip Monday tasks and run commitizen directly
      runCommitizen();
    }
  });
}

// Start the process
attachMondayTasksToCommit(); 
