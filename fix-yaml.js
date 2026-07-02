const fs = require('fs');
const path = require('path');

const tasksDir = path.join(__dirname, 'tasks');
const files = fs.readdirSync(tasksDir).filter(f => f.endsWith('.yaml'));

const topLevelKeys = [
  'id:', 'name:', 'description:', 'base_image:', 'setup_script:', 'prompt:', 'validation_script:'
];

for (const file of files) {
  const filePath = path.join(tasksDir, file);
  const content = fs.readFileSync(filePath, 'utf8');
  
  const lines = content.split('\n');
  const newLines = lines.map(line => {
    // If line is empty, return as is
    if (!line.trim()) return line;
    
    // If it starts with space, it's already indented
    if (line.startsWith(' ')) return line;
    
    // If it's a top level key, return as is
    if (topLevelKeys.some(key => line.startsWith(key))) return line;
    
    // Otherwise, it belongs to a multiline string from the previous key (like setup_script), so indent it!
    return '  ' + line;
  });
  
  fs.writeFileSync(filePath, newLines.join('\n'));
}

console.log(`Fixed indentation for ${files.length} yaml files.`);
