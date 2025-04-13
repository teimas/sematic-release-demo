module.exports = {
  types: [
    { value: 'feat', name: 'feat:     A new feature' },
    { value: 'fix', name: 'fix:      A bug fix' },
    { value: 'docs', name: 'docs:     Documentation only changes' },
    { value: 'style', name: 'style:    Code style changes (formatting, etc)' },
    { value: 'refactor', name: 'refactor: Code changes that neither fix bugs nor add features' },
    { value: 'perf', name: 'perf:     Performance improvements' },
    { value: 'test', name: 'test:     Adding or fixing tests' },
    { value: 'chore', name: 'chore:    Changes to the build process or auxiliary tools' },
    { value: 'revert', name: 'revert:   Revert to a commit' }
  ],
  messages: {
    type: 'Select the TYPE of change:',
    scope: 'Enter the SCOPE (PE.XX.XXX format):',
    subject: 'Enter a SHORT title:',
    body: 'Enter a DETAILED description (optional):',
    breaking: 'List any BREAKING CHANGES (optional):',
    footer: 'Additional Monday tasks (press Enter to skip):',
    confirmCommit: 'Proceed with the commit?'
  },
  allowCustomScopes: true,
  allowBreakingChanges: ['feat', 'fix'],
  subjectLimit: 100,
  upperCaseSubject: true,
  breakingPrefix: 'BREAKING CHANGE:',
  footerPrefix: 'MONDAY TASKS:',
  breaklineChar: '|',
  additionalQuestions: [
    {
      type: 'input',
      name: 'testDetails',
      message: 'Enter TEST details (optional, use | for new lines):',
      mapping: 'testDetails'
    },
    {
      type: 'input',
      name: 'security',
      message: 'Enter SECURITY considerations (use | for new lines, NA if not applicable):',
      mapping: 'security'
    },
    {
      type: 'input',
      name: 'references',
      message: 'Enter ticket reference (mXXXXXXXXXX format):',
      mapping: 'references'
    },
    {
      type: 'input',
      name: 'changeId',
      message: 'Enter Change-Id (will be auto-generated if empty):',
      mapping: 'changeId'
    }
  ],
  formatMessageCb: function (answers) {
    let msg = '';
    let mondayTasks = '';
    
    console.log('CHECKING', process.env.MONDAY_TASKS);
    // Try to get Monday tasks from environment variable first (backup approach)
    if (process.env.MONDAY_TASKS) {
      console.log('Found Monday tasks in environment variable');
      mondayTasks = process.env.MONDAY_TASKS;
    }
    
    // If not found in env, try reading from file
    if (!mondayTasks) {
      try {
        console.log('Reading Monday tasks file...');
        const fs = require('fs');
        const path = require('path');
        const tasksFilePath = path.resolve(__dirname, '.monday-tasks-temp');
        console.log('Tasks file path:', tasksFilePath);
        
        if (fs.existsSync(tasksFilePath)) {
          console.log('Tasks file exists, reading content...');
          mondayTasks = fs.readFileSync(tasksFilePath, 'utf8').trim();
          console.log('Monday tasks read from file:', mondayTasks);
        } else {
          console.log('Tasks file does not exist');
        }
      } catch (error) {
        console.error('Error reading Monday tasks:', error);
      }
    }

    // Format: Type(scope): Subject
    msg += `${answers.type}`;
    if (answers.scope) {
      msg += `(${answers.scope})`;
    }
    msg += `: ${answers.subject}`;

    // Add description
    if (answers.body) {
      msg += `\n\n${answers.body}`;
    }

    // Add breaking changes
    if (answers.breaking) {
      msg += `\n\nBREAKING CHANGE: ${answers.breaking}`;
    }

    // Add test details
    if (answers.testDetails) {
      msg += `\n\nTest Details: ${answers.testDetails}`;
    }

    // Add security (always include with default NA)
    const security = answers.security && answers.security.trim() ? answers.security : 'NA';
    msg += `\n\nSecurity: ${security}`;

    // Add refs
    if (answers.references) {
      msg += `\n\nRefs: ${answers.references}`;
    }

    // Add change ID
    if (answers.changeId) {
      msg += `\n\nChange-Id: ${answers.changeId}`;
    }

    // Add Monday tasks from temp file if available
    if (mondayTasks) {
      msg += `\n\nMONDAY TASKS: ${mondayTasks}`;
    }

    // Add additional Monday tasks if entered manually
    if (answers.footer && answers.footer.trim()) {
      if (mondayTasks) {
        msg += `, ${answers.footer}`;
      } else {
        msg += `\n\nMONDAY TASKS: ${answers.footer}`;
      }
    }
    
    console.log('Final commit message preview:');
    console.log(msg);
    return msg;
  }
};  