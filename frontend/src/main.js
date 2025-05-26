import { mount } from 'svelte'
import './app.css'
import App from './App.svelte'
import Login from './Login.svelte'
import { apiClient } from './js/api.js'
import { getStorageValue } from './js/storage'
import { initDeices } from './stores/devices'
import { initConversation } from './stores/conversation'

const initApp = async () => {
  const isAuthenticated = await getStorageValue('auth');

  if (isAuthenticated && apiClient.checkAuth()) {
    initDeices();
    initConversation();
    mount(App, {
      target: document.getElementById('app')
    })
  } else {
    mount(Login, {
      target: document.getElementById('app')
    });
  }
}

initApp()
