export function load() {
  return {
    secrets: [
      { id: 1, name: 'password', type: "login", value: '123456', security: "pk", description: 'password' },
      { id: 2, name: 'username', type: "note", value: 'admin', security: "pk", description: 'password' },
      { id: 3, name: 'email', type: "crypto key", value: 'asdfasdf', security: "pk", description: 'password' },
    ]
  };
}
