import { mount } from 'svelte'
import './app.css'
import App from './App.svelte'
import Login from './Login.svelte'
import { apiClient } from './js/api.js'
import { getStorageValue } from './js/storage'

const initApp = async () => {
  const isAuthenticated = await getStorageValue('auth');

  if (isAuthenticated && apiClient.checkAuth()) {
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
