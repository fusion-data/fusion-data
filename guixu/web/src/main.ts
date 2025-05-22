import { createApp } from 'vue'
import { createI18n } from 'vue-i18n'
import App from './App.vue'
import router from './router'

// import '@opentiny/vue/dist/index.css'
import './style.css'

const i18n = createI18n({
  legacy: false,
  locale: 'zh-CN',
  messages: {
    'zh-CN': {},
  },
})

const app = createApp(App)

app.use(i18n)
app.use(router)
app.mount('#app')
