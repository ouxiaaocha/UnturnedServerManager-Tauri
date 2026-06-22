// https://vitepress.dev/guide/custom-theme
import DefaultTheme from 'vitepress/theme'
import FeatureHero from './components/FeatureHero.vue'
import HomePage from './components/HomePage.vue'
import './style.css'

export default {
  extends: DefaultTheme,
  enhanceApp({ app }) {
    app.component('FeatureHero', FeatureHero)
    app.component('HomePage', HomePage)
  },
}
