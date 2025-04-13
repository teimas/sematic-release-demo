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
    footer: 'List any ISSUES CLOSED by this change (optional). E.g.: #31, #34:',
    confirmCommit: 'Proceed with the commit?'
  },
  allowCustomScopes: true,
  allowBreakingChanges: ['feat', 'fix'],
  subjectLimit: 100,
  upperCaseSubject: true,
  breakingPrefix: 'BREAKING CHANGE:',
  footerPrefix: 'REFERENCES:',
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
      msg += `\n\n${answers.breakingPrefix} ${answers.breaking}`;
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

    // Add footer
    if (answers.footer) {
      msg += `\n\n${answers.footerPrefix} ${answers.footer}`;
    }

    return msg;
  }
};  