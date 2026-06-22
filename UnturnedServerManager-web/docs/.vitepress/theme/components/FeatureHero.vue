<script setup lang="ts">
import { computed } from 'vue'

const props = defineProps<{
  eyebrow?: string
  title: string
  description: string
  icon?: string
  image?: string
  imageAlt?: string
  pills?: string
}>()

const pillItems = computed(() =>
  (props.pills ?? '')
    .split(',')
    .map((item) => item.trim())
    .filter(Boolean),
)
</script>

<template>
  <section class="feature-hero-panel">
    <div class="feature-hero-copy">
      <div class="feature-hero-kicker">
        <img v-if="icon" :src="icon" alt="" />
        <span>{{ eyebrow || 'Feature' }}</span>
      </div>
      <h1>{{ title }}</h1>
      <p>{{ description }}</p>
      <div v-if="pillItems.length" class="feature-hero-pills" aria-label="关键能力">
        <span v-for="pill in pillItems" :key="pill">{{ pill }}</span>
      </div>
    </div>
    <figure v-if="image" class="feature-hero-shot">
      <img :src="image" :alt="imageAlt || title" />
    </figure>
  </section>
</template>
