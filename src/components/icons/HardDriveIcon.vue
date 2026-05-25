<script setup lang="ts">
import IconBase from './IconBase.vue'

defineProps<{
  size?: number
}>()
</script>

<template>
  <IconBase :size="size">
    <path d="M2 12h20v6a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2v-6z" />
    <path d="M4 2h16a2 2 0 0 1 2 2v8H2V4a2 2 0 0 1 2-2z" />
    <circle cx="12" cy="15" r="2" />
  </IconBase>
</template>
