# Memory MCP Todo List PWA

A Progressive Web App (PWA) todo list application with local storage integration, built to demonstrate and test the Memory MCP server functionality.

## Features

- ✅ **Progressive Web App**: Installable, offline-capable, with service worker
- ✅ **Local Storage**: Persistent todo storage using browser localStorage
- ✅ **Modern UI**: Clean, responsive design with smooth animations
- ✅ **Todo Management**: Add, edit, delete, and mark todos as complete
- ✅ **Filtering**: Filter todos by All, Active, and Completed
- ✅ **Statistics**: Real-time todo statistics and progress tracking
- ✅ **PWA Features**: App manifest, service worker, install prompt

## Installation

### Option 1: Direct Access
1. Open `index.html` in a modern web browser
2. The app will work immediately with local storage

### Option 2: Install as PWA
1. Open the app in a supported browser (Chrome, Edge, Safari, Firefox)
2. Look for the install prompt or use the browser menu
3. Click "Install" to add to your home screen/desktop

### Option 3: Local Server
```bash
# Using Python
python -m http.server 8000

# Using Node.js
npx serve .

# Then visit http://localhost:8000/pwa-todo-app/
```

## Browser Support

- **Chrome/Edge**: Full PWA support
- **Firefox**: PWA support (install prompt may vary)
- **Safari**: PWA support on iOS/macOS
- **Mobile**: Full responsive support

## Technical Details

### Architecture
- **Frontend**: Vanilla JavaScript with modern ES6+ features
- **Storage**: Browser localStorage API
- **PWA**: Service Worker + Web App Manifest
- **Styling**: CSS with responsive design
- **No Dependencies**: Zero external JavaScript libraries

### Data Structure
```javascript
{
  id: "1640995200000",        // Unique timestamp-based ID
  text: "Buy groceries",       // Todo text content
  completed: false,            // Completion status
  createdAt: "2022-01-01T...", // ISO timestamp
  updatedAt: "2022-01-01T..."  // Last modified timestamp
}
```

### PWA Features
- **Service Worker**: Caches static assets, enables offline functionality
- **Web App Manifest**: Defines app metadata, icons, and install behavior
- **Install Prompt**: Automatic PWA install suggestion
- **Offline Support**: App works without internet connection

## Memory MCP Integration Testing

This PWA is designed to test the Memory MCP server's database operations:

### Test Scenarios
1. **Data Persistence**: Verify todos are stored and loaded correctly
2. **CRUD Operations**: Test Create, Read, Update, Delete operations
3. **Data Integrity**: Ensure data consistency across app restarts
4. **Performance**: Test storage/retrieval performance
5. **Error Handling**: Test storage failures and recovery

### MCP Server Testing
```bash
# Test memory queries
echo '{"jsonrpc":"2.0","id":1,"method":"tools/call","params":{"name":"query_memory","arguments":{"query":"todo","domain":"pwa","limit":10}}}' | cargo run --bin memory-mcp-server

# Test code execution
echo '{"jsonrpc":"2.0","id":2,"method":"tools/call","params":{"name":"execute_agent_code","arguments":{"code":"return localStorage.getItem(\"memory-mcp-todos\");","context":{"task":"test storage","input":{}}}}}' | cargo run --bin memory-mcp-server
```

## Development

### File Structure
```
pwa-todo-app/
├── index.html          # Main application
├── manifest.json       # PWA manifest
├── sw.js              # Service worker
└── README.md          # This file
```

### Local Development
```bash
# Start local server
python -m http.server 8000

# Open in browser
# http://localhost:8000/pwa-todo-app/

# Test PWA features
# - Try installing the app
# - Test offline functionality
# - Check local storage persistence
```

### Testing Checklist
- [ ] App loads without errors
- [ ] Add todo functionality works
- [ ] Todo persistence across page refreshes
- [ ] Filtering (All/Active/Completed) works
- [ ] Edit and delete functionality
- [ ] Clear completed works
- [ ] Statistics update correctly
- [ ] PWA install prompt appears
- [ ] App can be installed
- [ ] Offline functionality works
- [ ] Service worker is registered

## Memory MCP Database Verification

### Episode Creation Test
```bash
# Create episode for PWA testing
curl -X POST http://localhost:3000/episodes \
  -H "Content-Type: application/json" \
  -d '{
    "task_description": "Test PWA todo list with local storage",
    "context": {
      "domain": "pwa",
      "language": "javascript",
      "framework": "vanilla-js",
      "complexity": "simple"
    },
    "task_type": "CodeGeneration"
  }'
```

### Memory Query Test
```bash
# Query for PWA-related episodes
curl "http://localhost:3000/memory/query?text=pwa&domain=pwa&limit=5"
```

### Pattern Analysis Test
```bash
# Analyze patterns from PWA development
curl "http://localhost:3000/memory/patterns?task_type=CodeGeneration&limit=10"
```

## Contributing

1. Test all PWA features across different browsers
2. Verify local storage functionality
3. Test offline capabilities
4. Ensure responsive design works on mobile
5. Validate accessibility features

## License

MIT License - see project root for details.