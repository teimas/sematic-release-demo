#!/usr/bin/env node
const { spawn } = require('child_process');
const readline = require('readline');
const fs = require('fs');
const path = require('path');

// Path to the temporary file
const tasksFilePath = path.join(__dirname, '..', '.monday-tasks-temp');

// Create readline interface for user input
const rl = readline.createInterface({
  input: process.stdin,
  output: process.stdout
});

// Function to execute git-cz
function runCommitizen() {
  console.log('Starting commitizen...');
  
  // Try to read the tasks file and set as environment variable as backup approach
  try {
    if (fs.existsSync(tasksFilePath)) {
      const tasks = fs.readFileSync(tasksFilePath, 'utf8').trim();
      if (tasks) {
        console.log('Setting tasks to environment variable as backup approach');
        process.env.MONDAY_TASKS = tasks;
        console.log(process.env.MONDAY_TASKS);
      }
    }
  } catch (error) {
    console.error('Error setting environment variable:', error);
  }
  
  // Spawn git-cz process
  const gitCz = spawn('npx', ['git-cz'], { 
    stdio: 'inherit',
    shell: true,
    env: { ...process.env } // Pass all environment variables including our new one
  });

  // Handle process completion
  gitCz.on('close', (code) => {
    // Clean up the temporary file after commit is done
    try {
      if (fs.existsSync(tasksFilePath)) {
        fs.unlinkSync(tasksFilePath);
        console.log('Cleaned up temporary files');
      }
    } catch (error) {
      console.error('Error cleaning up temporary files:', error);
    }
    
    process.exit(code);
  });
}

// Function to check for Monday tasks when footer is requested
function attachMondayTasksToCommit() {
  // Make sure the temporary file doesn't exist from a previous run
  try {
    if (fs.existsSync(tasksFilePath)) {
      fs.unlinkSync(tasksFilePath);
    }
  } catch (error) {
    console.error('Error cleaning up existing temporary file:', error);
  }
  
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
