<script setup lang="ts">
import IconBase from './IconBase.vue'

defineProps<{
  size?: number
}>()
</script>

<template>
  <IconBase :size="size">
    <ellipse cx="12" cy="14" rx="5" ry="6" />
    <circle cx="6" cy="8" r="2" />
    <circle cx="18" cy="8" r="2" />
    <circle cx="8" cy="4" r="2" />
    <circle cx="16" cy="4" r="2" />
  </IconBase>
</template>
