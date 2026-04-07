import { mount } from 'svelte';
import App from './App.svelte';
import { logError } from './lib/ipc';
import './app.css';

window.addEventListener('error', (event) => {
  const message = event.error?.message ?? event.message ?? 'Unknown error';
  const stack = event.error?.stack;
  void logError(message, stack);
});

window.addEventListener('unhandledrejection', (event) => {
  const reason = event.reason;
  const message = reason instanceof Error ? reason.message : String(reason);
  const stack = reason instanceof Error ? reason.stack : undefined;
  void logError(`Unhandled rejection: ${message}`, stack);
});

// noinspection TypeScriptValidateTypes
const app = mount(App, { target: document.getElementById('app')! });

export default app;
