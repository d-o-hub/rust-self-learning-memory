// Service Worker for Memory MCP Todo PWA
const CACHE_NAME = 'memory-mcp-todo-v1';
const STATIC_CACHE = 'memory-mcp-todo-static-v1';
const DYNAMIC_CACHE = 'memory-mcp-todo-dynamic-v1';

// Files to cache immediately
const STATIC_FILES = [
    '/pwa-todo-app/',
    '/pwa-todo-app/index.html',
    '/pwa-todo-app/manifest.json',
    // Add more static assets here as needed
];

// Install event - cache static files
self.addEventListener('install', event => {
    console.log('[Service Worker] Installing');
    event.waitUntil(
        caches.open(STATIC_CACHE)
            .then(cache => {
                console.log('[Service Worker] Caching static files');
                return cache.addAll(STATIC_FILES);
            })
            .then(() => {
                return self.skipWaiting();
            })
    );
});

// Activate event - clean up old caches
self.addEventListener('activate', event => {
    console.log('[Service Worker] Activating');
    event.waitUntil(
        caches.keys()
            .then(cacheNames => {
                return Promise.all(
                    cacheNames.map(cacheName => {
                        if (cacheName !== STATIC_CACHE && cacheName !== DYNAMIC_CACHE) {
                            console.log('[Service Worker] Deleting old cache:', cacheName);
                            return caches.delete(cacheName);
                        }
                    })
                );
            })
            .then(() => {
                return self.clients.claim();
            })
    );
});

// Fetch event - serve from cache or network
self.addEventListener('fetch', event => {
    const { request } = event;
    const url = new URL(request.url);

    // Only handle GET requests
    if (request.method !== 'GET') return;

    // Handle different types of requests
    if (url.pathname.startsWith('/pwa-todo-app/')) {
        // App-specific requests - try cache first, then network
        event.respondWith(
            caches.match(request)
                .then(response => {
                    if (response) {
                        return response;
                    }

                    return fetch(request)
                        .then(response => {
                            // Don't cache if not a valid response
                            if (!response || response.status !== 200 || response.type !== 'basic') {
                                return response;
                            }

                            // Clone the response for caching
                            const responseToCache = response.clone();

                            caches.open(DYNAMIC_CACHE)
                                .then(cache => {
                                    cache.put(request, responseToCache);
                                });

                            return response;
                        })
                        .catch(() => {
                            // Return offline fallback if available
                            if (request.destination === 'document') {
                                return caches.match('/pwa-todo-app/index.html');
                            }
                        });
                })
        );
    }
});

// Background sync for when connection is restored
self.addEventListener('sync', event => {
    console.log('[Service Worker] Background sync triggered:', event.tag);

    if (event.tag === 'todo-sync') {
        event.waitUntil(syncTodos());
    }
});

// Push notifications (for future enhancement)
self.addEventListener('push', event => {
    console.log('[Service Worker] Push received:', event);

    if (event.data) {
        const data = event.data.json();
        const options = {
            body: data.body,
            icon: '/pwa-todo-app/icon-192x192.png',
            badge: '/pwa-todo-app/icon-192x192.png',
            vibrate: [100, 50, 100],
            data: {
                dateOfArrival: Date.now(),
                primaryKey: data.primaryKey
            }
        };

        event.waitUntil(
            self.registration.showNotification(data.title, options)
        );
    }
});

// Notification click handler
self.addEventListener('notificationclick', event => {
    console.log('[Service Worker] Notification clicked:', event);
    event.notification.close();

    event.waitUntil(
        clients.openWindow('/pwa-todo-app/')
    );
});

// Sync function for background sync
async function syncTodos() {
    try {
        console.log('[Service Worker] Syncing todos...');

        // Get all clients (tabs/windows)
        const clients = await self.clients.matchAll();

        // Send sync message to all clients
        clients.forEach(client => {
            client.postMessage({
                type: 'SYNC_TODOS',
                timestamp: Date.now()
            });
        });

        console.log('[Service Worker] Todo sync completed');
    } catch (error) {
        console.error('[Service Worker] Todo sync failed:', error);
    }
}

// Message handler for communication with clients
self.addEventListener('message', event => {
    console.log('[Service Worker] Message received:', event.data);

    if (event.data && event.data.type === 'SKIP_WAITING') {
        self.skipWaiting();
    }

    if (event.data && event.data.type === 'GET_VERSION') {
        event.ports[0].postMessage({
            version: '1.0.0',
            cache: CACHE_NAME
        });
    }
});